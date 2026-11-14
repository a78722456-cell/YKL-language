use apl_ast::{BinaryOp, EnumDecl, StructDecl, Type, UnaryOp};
use apl_hir::FunctionId;
use apl_mir::{
    Constant, LocalDecl, MirBody, Operand, Place, Rvalue, Statement, Terminator,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CodegenFunction {
    pub id: FunctionId,
    pub name: String,
    pub return_type: Type,
    pub body: MirBody,
}

#[derive(Debug, Clone, Default)]
pub struct CCodeGenerator {
    pub bare_metal: bool,
}

impl CCodeGenerator {
    pub fn new() -> Self {
        Self { bare_metal: false }
    }

    pub fn with_bare_metal(mut self, bare_metal: bool) -> Self {
        self.bare_metal = bare_metal;
        self
    }
    fn c_type(ty: &Type) -> String {
        match ty {
            Type::I8 => "int8_t".to_string(),
            Type::I16 => "int16_t".to_string(),
            Type::I32 => "int32_t".to_string(),
            Type::I64 | Type::ISize => "int64_t".to_string(),
            Type::U8 => "uint8_t".to_string(),
            Type::U16 => "uint16_t".to_string(),
            Type::U32 => "uint32_t".to_string(),
            Type::U64 | Type::USize => "uint64_t".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "const char*".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(elem, _) => Self::c_type(elem),
            Type::Named(name, _) => format!("struct {}", name),
            Type::Ref(inner) | Type::RefMut(inner) => {
                if let Type::Named(name, _) = &**inner {
                    format!("struct {}*", name)
                } else {
                    format!("{}*", Self::c_type(inner))
                }
            }
            _ => "int64_t".to_string(),
        }
    }

    fn escape_c_string(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    fn format_place(
        place: &Place,
        locals: &[LocalDecl],
        structs: &HashMap<String, StructDecl>,
    ) -> String {
        match place {
            Place::Local(id) => format!("_l{}", id.0),
            Place::Field { base, field } => {
                let base_str = Self::format_place(base, locals, structs);
                let base_ty = Self::place_type(base, locals);
                let is_ptr = matches!(base_ty, Type::Ref(_) | Type::RefMut(_));
                let op = if is_ptr { "->" } else { "." };

                if let Some(struct_name) = Self::place_struct_name(base, locals) {
                    if let Some(st) = structs.get(&struct_name) {
                        if let Some(f_decl) = st.fields.get(field.0 as usize) {
                            return format!("{}{}{}", base_str, op, f_decl.name);
                        }
                    }
                    if field.0 == 0 {
                        return format!("{}{}{}", base_str, op, "tag");
                    } else {
                        return format!("{}{}{}", base_str, op, format!("data._payload[{}]", field.0 - 1));
                    }
                }
                if field.0 == 0 {
                    format!("{}{}{}", base_str, op, "tag")
                } else {
                    format!("{}{}{}", base_str, op, format!("f{}", field.0))
                }
            }
            Place::Index { base, index } => {
                format!(
                    "{}[_l{}]",
                    Self::format_place(base, locals, structs),
                    index.0
                )
            }
            Place::Deref { base } => format!("(*{})", Self::format_place(base, locals, structs)),
        }
    }

    fn place_type(place: &Place, locals: &[LocalDecl]) -> Type {
        match place {
            Place::Local(id) => locals
                .iter()
                .find(|l| l.id == *id)
                .map(|l| l.ty.clone())
                .unwrap_or(Type::I32),
            Place::Deref { base } => match Self::place_type(base, locals) {
                Type::Ref(inner) | Type::RefMut(inner) => *inner,
                other => other,
            },
            Place::Field { base, field: _ } => {
                let base_ty = Self::place_type(base, locals);
                match base_ty {
                    Type::Ref(inner) | Type::RefMut(inner) => match *inner {
                        Type::Named(ref name, _) => Type::Named(name.clone(), vec![]),
                        other => other,
                    },
                    other => other,
                }
            }
            _ => Type::I32,
        }
    }

    fn place_struct_name(place: &Place, locals: &[LocalDecl]) -> Option<String> {
        match Self::place_type(place, locals) {
            Type::Named(name, _) => Some(name),
            Type::Ref(inner) | Type::RefMut(inner) => match *inner {
                Type::Named(name, _) => Some(name),
                _ => None,
            },
            _ => None,
        }
    }

    fn operand_type(op: &Operand, locals: &[LocalDecl]) -> Type {
        match op {
            Operand::Copy(p) | Operand::Move(p) => Self::place_type(p, locals),
            Operand::Constant(c) => match c {
                Constant::Int(_, ty) => ty.clone(),
                Constant::Float(_, ty) => ty.clone(),
                Constant::Bool(_) => Type::Bool,
                Constant::String(_) => Type::String,
            },
        }
    }

    fn format_operand(
        op: &Operand,
        locals: &[LocalDecl],
        structs: &HashMap<String, StructDecl>,
    ) -> String {
        match op {
            Operand::Copy(p) | Operand::Move(p) => Self::format_place(p, locals, structs),
            Operand::Constant(c) => match c {
                Constant::Int(v, _) => format!("{}LL", v),
                Constant::Float(v, _) => format!("{:?}", v),
                Constant::Bool(b) => if *b { "true" } else { "false" }.to_string(),
                Constant::String(s) => Self::escape_c_string(s),
            },
        }
    }

    pub fn generate_program(
        functions: &[CodegenFunction],
        structs: &HashMap<String, StructDecl>,
        enums: &HashMap<String, EnumDecl>,
        _traits: &[()],
        bare_metal: bool,
    ) -> String {
        Self::new().with_bare_metal(bare_metal).generate_c(functions, structs, enums)
    }

    pub fn generate_c(
        &self,
        functions: &[CodegenFunction],
        structs: &HashMap<String, StructDecl>,
        enums: &HashMap<String, EnumDecl>,
    ) -> String {
        let mut code = String::new();
        code.push_str("/* Generated by Young Kazah Language (YKL / APL) Compiler */\n");
        if self.bare_metal {
            code.push_str("/* Bare-Metal Freestanding Target (No OS, no libc required) */\n");
            code.push_str("#include <stdint.h>\n");
            code.push_str("#include <stdbool.h>\n\n");
        } else {
            code.push_str("#include <stdio.h>\n");
            code.push_str("#include <stdlib.h>\n");
            code.push_str("#include <stdint.h>\n");
            code.push_str("#include <stdbool.h>\n");
            code.push_str("#include <string.h>\n\n");
        }

        // 1. Предварительные объявления структур и enum
        for st_name in structs.keys() {
            code.push_str(&format!("struct {};\n", st_name));
        }
        for en_name in enums.keys() {
            code.push_str(&format!("struct {};\n", en_name));
        }
        code.push('\n');

        // 2. Определения enums (Tagged Unions)
        for (en_name, en_decl) in enums {
            code.push_str(&format!("enum {}_Tag {{\n", en_name));
            for (idx, var) in en_decl.variants.iter().enumerate() {
                code.push_str(&format!("    {}_{} = {},\n", en_name, var.name, idx));
            }
            code.push_str("};\n");
            code.push_str(&format!("struct {} {{\n", en_name));
            code.push_str(&format!("    enum {}_Tag tag;\n", en_name));
            code.push_str("    union {\n");
            for var in &en_decl.variants {
                if !var.types.is_empty() {
                    code.push_str("        struct { ");
                    for (fi, f_ty) in var.types.iter().enumerate() {
                        code.push_str(&format!("{} f{}; ", Self::c_type(f_ty), fi));
                    }
                    code.push_str(&format!("}} {};\n", var.name));
                }
            }
            code.push_str("        int64_t _payload[4];\n");
            code.push_str("    } data;\n};\n\n");
        }

        // 3. Полные определения структур (включая @packed и fixed arrays)
        for (st_name, st_decl) in structs {
            if st_decl.is_packed {
                code.push_str(&format!("struct __attribute__((packed)) {} {{\n", st_name));
            } else {
                code.push_str(&format!("struct {} {{\n", st_name));
            }
            if st_decl.fields.is_empty() {
                code.push_str("    uint8_t _dummy;\n");
            } else {
                for field in &st_decl.fields {
                    if let Type::Array(elem, len) = &field.ty {
                        code.push_str(&format!(
                            "    {} {}[{}];\n",
                            Self::c_type(elem),
                            field.name,
                            len
                        ));
                    } else {
                        code.push_str(&format!(
                            "    {} {};\n",
                            Self::c_type(&field.ty),
                            field.name
                        ));
                    }
                }
            }
            code.push_str("};\n\n");
        }

        // Карта имен, типов возврата и типов параметров функций по FunctionId
        let mut func_signatures: HashMap<FunctionId, (String, Type, Vec<Type>)> = HashMap::new();
        for f in functions {
            let c_name = if f.name == "main" {
                "ykl_main".to_string()
            } else {
                f.name.replace("::", "_")
            };
            let param_tys: Vec<Type> = f
                .body
                .locals
                .iter()
                .filter(|l| l.is_param)
                .map(|l| l.ty.clone())
                .collect();
            func_signatures.insert(f.id, (c_name, f.return_type.clone(), param_tys));
        }

        // 3. Прототипы функций
        for f in functions {
            let (c_name, _, _) = &func_signatures[&f.id];
            let ret_c = Self::c_type(&f.return_type);
            let params: Vec<String> = f
                .body
                .locals
                .iter()
                .filter(|l| l.is_param)
                .map(|l| format!("{} _l{}", Self::c_type(&l.ty), l.id.0))
                .collect();
            let params_sig = if params.is_empty() {
                "void".to_string()
            } else {
                params.join(", ")
            };
            code.push_str(&format!("{} {}({});\n", ret_c, c_name, params_sig));
        }
        code.push('\n');

        // 4. Определения функций
        for f in functions {
            let (c_name, _, _) = &func_signatures[&f.id];
            let ret_c = Self::c_type(&f.return_type);
            let params: Vec<String> = f
                .body
                .locals
                .iter()
                .filter(|l| l.is_param)
                .map(|l| format!("{} _l{}", Self::c_type(&l.ty), l.id.0))
                .collect();
            let params_sig = if params.is_empty() {
                "void".to_string()
            } else {
                params.join(", ")
            };

            code.push_str(&format!("{} {}({}) {{\n", ret_c, c_name, params_sig));

            // Объявления локальных переменных (не параметров и не Void)
            for local in &f.body.locals {
                if !local.is_param && local.ty != Type::Void {
                    if let Type::Array(elem, len) = &local.ty {
                        code.push_str(&format!(
                            "    {} _l{}[{}] = {{0}};\n",
                            Self::c_type(elem),
                            local.id.0,
                            len
                        ));
                    } else {
                        let zero_init = match &local.ty {
                            Type::Named(name, _) => format!("(struct {}){{0}}", name),
                            Type::Ref(_) | Type::RefMut(_) => "NULL".to_string(),
                            _ => "0".to_string(),
                        };
                        code.push_str(&format!(
                            "    {} _l{} = {};\n",
                            Self::c_type(&local.ty),
                            local.id.0,
                            zero_init
                        ));
                    }
                }
            }
            code.push('\n');

            // Генерация базовых блоков CFG
            for (idx, block) in f.body.blocks.iter().enumerate() {
                code.push_str(&format!("bb{}:\n", idx));
                for stmt in &block.statements {
                    if let Statement::Assign(place, rvalue, _) = stmt {
                        let place_ty = Self::place_type(place, &f.body.locals);
                        if place_ty == Type::Void {
                            continue;
                        }
                        let lhs = Self::format_place(place, &f.body.locals, structs);
                        if let Rvalue::Aggregate(fields) = rvalue {
                            if let Type::Array(_, _) = &place_ty {
                                for (i, op) in fields.iter().enumerate() {
                                    let val = Self::format_operand(op, &f.body.locals, structs);
                                    code.push_str(&format!("    {}[{}] = {};\n", lhs, i, val));
                                }
                                continue;
                            }
                            if let Type::Named(en_name, _) = &place_ty {
                                if let Some(_en_decl) = enums.get(en_name) {
                                    if let Some(tag_op) = fields.first() {
                                        let tag_val = Self::format_operand(tag_op, &f.body.locals, structs);
                                        code.push_str(&format!("    {}.tag = {};\n", lhs, tag_val));
                                        if fields.len() > 1 {
                                            for (pi, p_op) in fields[1..].iter().enumerate() {
                                                let p_val = Self::format_operand(p_op, &f.body.locals, structs);
                                                code.push_str(&format!("    {}.data._payload[{}] = {};\n", lhs, pi, p_val));
                                            }
                                        }
                                    }
                                    continue;
                                }
                            }
                        }
                        let rhs = match rvalue {
                            Rvalue::Use(op) => Self::format_operand(op, &f.body.locals, structs),
                            Rvalue::Ref(_, _, p) => {
                                let p_ty = Self::place_type(p, &f.body.locals);
                                if matches!(p_ty, Type::Ref(_) | Type::RefMut(_)) {
                                    Self::format_place(p, &f.body.locals, structs)
                                } else {
                                    format!("&{}", Self::format_place(p, &f.body.locals, structs))
                                }
                            }
                            Rvalue::BinaryOp(op, l, r) => {
                                let ls = Self::format_operand(l, &f.body.locals, structs);
                                let rs = Self::format_operand(r, &f.body.locals, structs);
                                let op_str = match op {
                                    BinaryOp::Add => "+",
                                    BinaryOp::Sub => "-",
                                    BinaryOp::Mul => "*",
                                    BinaryOp::Div => "/",
                                    BinaryOp::Mod => "%",
                                    BinaryOp::Eq => "==",
                                    BinaryOp::Neq => "!=",
                                    BinaryOp::Lt => "<",
                                    BinaryOp::Lte => "<=",
                                    BinaryOp::Gt => ">",
                                    BinaryOp::Gte => ">=",
                                    BinaryOp::And => "&&",
                                    BinaryOp::Or => "||",
                                    BinaryOp::BitAnd => "&",
                                    BinaryOp::BitOr => "|",
                                    BinaryOp::BitXor => "^",
                                    BinaryOp::Shl => "<<",
                                    BinaryOp::Shr => ">>",
                                };
                                format!("({} {} {})", ls, op_str, rs)
                            }
                            Rvalue::UnaryOp(op, operand) => {
                                let v = Self::format_operand(operand, &f.body.locals, structs);
                                match op {
                                    UnaryOp::Neg => format!("(-{})", v),
                                    UnaryOp::Not => format!("(!{})", v),
                                    UnaryOp::Deref => format!("(*{})", v),
                                    UnaryOp::Ref | UnaryOp::RefMut => format!("&{}", v),
                                    _ => v,
                                }
                            }
                            Rvalue::Aggregate(fields) => {
                                let vals: Vec<String> = fields
                                    .iter()
                                    .map(|op| Self::format_operand(op, &f.body.locals, structs))
                                    .collect();
                                format!("({}) {{ {} }}", Self::c_type(&place_ty), vals.join(", "))
                            }
                        };
                        code.push_str(&format!("    {} = {};\n", lhs, rhs));
                    }
                }

                // Терминатор блока
                match &block.terminator {
                    Terminator::Goto(target) => {
                        code.push_str(&format!("    goto bb{};\n", target.0));
                    }
                    Terminator::SwitchInt {
                        discriminant,
                        targets,
                        otherwise,
                    } => {
                        let disc = Self::format_operand(discriminant, &f.body.locals, structs);
                        for (val, target) in targets {
                            code.push_str(&format!(
                                "    if ({} == {}) goto bb{};\n",
                                disc, val, target.0
                            ));
                        }
                        code.push_str(&format!("    goto bb{};\n", otherwise.0));
                    }
                    Terminator::Call {
                        function,
                        args,
                        destination,
                        target,
                        ..
                    } => {
                        match function.0 {
                            99010 => {
                                let addr = Self::format_operand(&args[0], &f.body.locals, structs);
                                let val = Self::format_operand(&args[1], &f.body.locals, structs);
                                code.push_str(&format!("    *((volatile uint32_t*)(uintptr_t)({})) = (uint32_t)({});\n", addr, val));
                                code.push_str(&format!("    goto bb{};\n", target.0));
                                continue;
                            }
                            99011 => {
                                let addr = Self::format_operand(&args[0], &f.body.locals, structs);
                                let dest = Self::format_place(destination, &f.body.locals, structs);
                                code.push_str(&format!("    {} = (int32_t)(*((volatile uint32_t*)(uintptr_t)({})));\n", dest, addr));
                                code.push_str(&format!("    goto bb{};\n", target.0));
                                continue;
                            }
                            99012 => {
                                let addr = Self::format_operand(&args[0], &f.body.locals, structs);
                                let bit = Self::format_operand(&args[1], &f.body.locals, structs);
                                code.push_str(&format!("    *((volatile uint32_t*)(uintptr_t)({})) |= (1UL << ({}));\n", addr, bit));
                                code.push_str(&format!("    goto bb{};\n", target.0));
                                continue;
                            }
                            99013 => {
                                let addr = Self::format_operand(&args[0], &f.body.locals, structs);
                                let bit = Self::format_operand(&args[1], &f.body.locals, structs);
                                code.push_str(&format!("    *((volatile uint32_t*)(uintptr_t)({})) &= ~(1UL << ({}));\n", addr, bit));
                                code.push_str(&format!("    goto bb{};\n", target.0));
                                continue;
                            }
                            99000 | 99001 | 99002 => {
                                if self.bare_metal {
                                    code.push_str("    /* IO disabled in bare-metal mode */\n");
                                    code.push_str(&format!("    goto bb{};\n", target.0));
                                    continue;
                                }
                                let stream = if function.0 == 99002 { "stderr" } else { "stdout" };
                                let newline = if function.0 == 99000 { "" } else { "\\n" };
                                if args.is_empty() {
                                    if !newline.is_empty() {
                                        code.push_str(&format!("    fprintf({}, \"\\n\");\n", stream));
                                    }
                                } else {
                                    for (i, arg) in args.iter().enumerate() {
                                        let arg_ty = Self::operand_type(arg, &f.body.locals);
                                        let arg_c = Self::format_operand(arg, &f.body.locals, structs);
                                        let sep = if i + 1 < args.len() { " " } else { newline };
                                        match arg_ty {
                                            Type::String => code.push_str(&format!(
                                                "    fprintf({}, \"%s{}\", {});\n",
                                                stream, sep, arg_c
                                            )),
                                            Type::F32 | Type::F64 => code.push_str(&format!(
                                                "    fprintf({}, \"%g{}\", (double)({}));\n",
                                                stream, sep, arg_c
                                            )),
                                            Type::Bool => code.push_str(&format!(
                                                "    fprintf({}, \"%s{}\", ({}) ? \"true\" : \"false\");\n",
                                                stream, sep, arg_c
                                            )),
                                            _ => code.push_str(&format!(
                                                "    fprintf({}, \"%lld{}\", (long long)({}));\n",
                                                stream, sep, arg_c
                                            )),
                                        }
                                    }
                                }
                            }
                            _ => {
                                if let Some((callee_name, ret_ty, param_types)) = func_signatures.get(function) {
                                    let mut arg_strs: Vec<String> = Vec::new();
                                    for (i, arg) in args.iter().enumerate() {
                                        let arg_ty = Self::operand_type(arg, &f.body.locals);
                                        let mut arg_c = Self::format_operand(arg, &f.body.locals, structs);
                                        if let Some(expected_ty) = param_types.get(i) {
                                            if matches!(expected_ty, Type::Ref(_) | Type::RefMut(_)) && !matches!(arg_ty, Type::Ref(_) | Type::RefMut(_)) {
                                                arg_c = format!("&{}", arg_c);
                                            }
                                        }
                                        arg_strs.push(arg_c);
                                    }
                                    let call_str = format!("{}({})", callee_name, arg_strs.join(", "));
                                    let dest_ty = Self::place_type(destination, &f.body.locals);
                                    if *ret_ty != Type::Void && dest_ty != Type::Void {
                                        let lhs = Self::format_place(destination, &f.body.locals, structs);
                                        code.push_str(&format!("    {} = {};\n", lhs, call_str));
                                    } else {
                                        code.push_str(&format!("    {};\n", call_str));
                                    }
                                }
                            }
                        }
                        code.push_str(&format!("    goto bb{};\n", target.0));
                    }
                    Terminator::Return(_) => {
                        if f.return_type == Type::Void {
                            code.push_str("    return;\n");
                        } else {
                            code.push_str("    return _l0;\n");
                        }
                    }
                    Terminator::Drop { target, .. } => {
                        code.push_str(&format!("    goto bb{};\n", target.0));
                    }
                    Terminator::Unreachable | Terminator::Resume => {
                        if self.bare_metal {
                            code.push_str("    while (1) {}\n");
                        } else {
                            code.push_str("    abort();\n");
                        }
                    }
                }
            }
            code.push_str("}\n\n");
        }

        // 5. Точка входа main() на C
        if self.bare_metal {
            code.push_str("/* Freestanding Bare-Metal entry point */\n");
            code.push_str("void Reset_Handler(void) {\n");
            code.push_str("    ykl_main();\n");
            code.push_str("    while (1) {}\n");
            code.push_str("}\n");
        } else {
            code.push_str("int main(int argc, char** argv) {\n");
            code.push_str("    (void)argc; (void)argv;\n");
            if let Some(main_fn) = functions.iter().find(|f| f.name == "main") {
                if main_fn.return_type == Type::Void {
                    code.push_str("    ykl_main();\n    return 0;\n");
                } else {
                    code.push_str("    return (int)ykl_main();\n");
                }
            } else {
                code.push_str("    return 0;\n");
            }
            code.push_str("}\n");
        }

        code
    }
}

use apl_ast::*;

pub struct Formatter {
    indent_level: usize,
    output: String,
}

impl Formatter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn format_program(&mut self, prog: &Program) -> String {
        for item in &prog.items {
            self.format_item(item);
            self.output.push_str("\n\n");
        }
        self.output.trim_end().to_string() + "\n"
    }

    fn push_indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent_level));
    }

    fn format_item(&mut self, item: &Item) {
        match item {
            Item::Function(func) => self.format_function(func),
            Item::Struct(st) => self.format_struct(st),
            Item::Enum(en) => self.format_enum(en),
            Item::ExternBlock(decls, _) => self.format_extern_block(decls, false),
            Item::Trait(tr) => self.format_trait(tr),
            Item::Impl(imp) => self.format_impl(imp),
            Item::Import(path, _) => {
                self.output.push_str(&format!("import \"{}\";", path));
            }
            Item::Package(name, _) => {
                self.output.push_str(&format!("package {};", name));
            }
        }
    }

    fn format_function(&mut self, func: &Function) {
        if func.is_pub {
            self.output.push_str("pub ");
        }
        if func.is_async {
            self.output.push_str("async ");
        }
        self.output.push_str(&format!("fn {}(", func.name));
        for (i, param) in func.params.iter().enumerate() {
            self.output.push_str(&format!("{}: {}", param.name, self.format_type(&param.ty)));
            if i < func.params.len() - 1 {
                self.output.push_str(", ");
            }
        }
        self.output.push_str(&format!(") -> {} ", self.format_type(&func.return_type)));
        self.format_expr(&func.body);
    }

    fn format_struct(&mut self, st: &StructDecl) {
        if st.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str(&format!("struct {} {{\n", st.name));
        self.indent_level += 1;
        for field in &st.fields {
            self.push_indent();
            self.output.push_str(&format!("{}: {},\n", field.name, self.format_type(&field.ty)));
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}");
    }

    fn format_enum(&mut self, en: &EnumDecl) {
        if en.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str(&format!("enum {} {{\n", en.name));
        self.indent_level += 1;
        for variant in &en.variants {
            self.push_indent();
            self.output.push_str(&format!("{}", variant.name));
            if !variant.types.is_empty() {
                self.output.push_str("(");
                for (i, ty) in variant.types.iter().enumerate() {
                    self.output.push_str(&self.format_type(ty));
                    if i < variant.types.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
            }
            self.output.push_str(",\n");
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}");
    }

    fn format_extern_block(&mut self, decls: &[ExternDecl], is_pub: bool) {
        if is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str("extern \"C\" {\n");
        self.indent_level += 1;
        for decl in decls {
            self.push_indent();
            self.output.push_str(&format!("fn {}(", decl.name));
            for (i, param) in decl.params.iter().enumerate() {
                self.output.push_str(&format!("{}: {}", param.0, self.format_type(&param.1)));
                if i < decl.params.len() - 1 {
                    self.output.push_str(", ");
                }
            }
            self.output.push_str(&format!(") -> {};\n", self.format_type(&decl.return_type)));
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}");
    }

    fn format_trait(&mut self, tr: &TraitDecl) {
        if tr.is_pub {
            self.output.push_str("pub ");
        }
        self.output.push_str(&format!("trait {} {{\n", tr.name));
        self.indent_level += 1;
        for method in &tr.methods {
            self.push_indent();
            self.output.push_str(&format!("fn {}(", method.name));
            for (i, param) in method.params.iter().enumerate() {
                self.output.push_str(&format!("{}: {}", param.name, self.format_type(&param.ty)));
                if i < method.params.len() - 1 {
                    self.output.push_str(", ");
                }
            }
            self.output.push_str(&format!(") -> {};\n", self.format_type(&method.return_type)));
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}");
    }

    fn format_impl(&mut self, imp: &ImplDecl) {
        if let Some(trait_name) = &imp.trait_name {
            self.output.push_str(&format!("impl {} for {} {{\n", trait_name, imp.target_type));
        } else {
            self.output.push_str(&format!("impl {} {{\n", imp.target_type));
        }
        self.indent_level += 1;
        for method in &imp.methods {
            self.push_indent();
            self.format_function(method);
            self.output.push_str("\n");
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}");
    }

    fn format_type(&self, ty: &Type) -> String {
        format!("{}", ty)
    }
    
    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(e) => {
                self.format_expr(e);
            }
            Stmt::Let { name, mutability, ty, init, span: _ } => {
                self.output.push_str("let ");
                if *mutability {
                    self.output.push_str("mut ");
                }
                self.output.push_str(name);
                if let Some(t) = ty {
                    self.output.push_str(&format!(": {}", self.format_type(t)));
                }
                self.output.push_str(" = ");
                self.format_expr(init);
            }
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => {
                match lit {
                    Literal::Int(v) => self.output.push_str(&v.to_string()),
                    Literal::Float(v) => self.output.push_str(&v.to_string()),
                    Literal::Bool(v) => self.output.push_str(&v.to_string()),
                    Literal::String(v) => self.output.push_str(&format!("\"{}\"", v)),
                    Literal::Char(v) => self.output.push_str(&format!("'{}'", v)),
                }
            }
            ExprKind::Ident(v) => self.output.push_str(v),
            ExprKind::Binary(op, left, right) => {
                self.format_expr(left);
                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Mod => "%",
                    BinaryOp::Eq => "==",
                    BinaryOp::Neq => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::Lte => "<=",
                    BinaryOp::Gte => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Shl => "<<",
                    BinaryOp::Shr => ">>",
                };
                self.output.push_str(&format!(" {} ", op_str));
                self.format_expr(right);
            }
            ExprKind::Unary(op, right) => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::Ref => "&",
                    UnaryOp::RefMut => "&mut ",
                    UnaryOp::Deref => "*",
                    UnaryOp::TryUnwrap => "?",
                };
                self.output.push_str(op_str);
                self.format_expr(right);
            }
            ExprKind::Call(callee, args) => {
                self.format_expr(callee);
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    self.format_expr(arg);
                    if i < args.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
            }
            ExprKind::MethodCall(obj, method, args) => {
                self.format_expr(obj);
                self.output.push_str(&format!(".{}(", method));
                for (i, arg) in args.iter().enumerate() {
                    self.format_expr(arg);
                    if i < args.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(")");
            }
            ExprKind::FieldAccess(obj, field) => {
                self.format_expr(obj);
                self.output.push_str(&format!(".{}", field));
            }
            ExprKind::StructInit(name, fields) => {
                self.output.push_str(&format!("{} {{ ", name));
                for (i, (fname, fexpr)) in fields.iter().enumerate() {
                    self.output.push_str(&format!("{}: ", fname));
                    self.format_expr(fexpr);
                    if i < fields.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str(" }");
            }
            ExprKind::EnumVariant(name, variant, args) => {
                self.output.push_str(&format!("{}::{}", name, variant));
                if !args.is_empty() {
                    self.output.push_str("(");
                    for (i, arg) in args.iter().enumerate() {
                        self.format_expr(arg);
                        if i < args.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push_str(")");
                }
            }
            ExprKind::ArrayInit(elements) => {
                self.output.push_str("[");
                for (i, arg) in elements.iter().enumerate() {
                    self.format_expr(arg);
                    if i < elements.len() - 1 {
                        self.output.push_str(", ");
                    }
                }
                self.output.push_str("]");
            }
            ExprKind::Index(arr, idx) => {
                self.format_expr(arr);
                self.output.push_str("[");
                self.format_expr(idx);
                self.output.push_str("]");
            }
            ExprKind::Assign(left, right) => {
                self.format_expr(left);
                self.output.push_str(" = ");
                self.format_expr(right);
            }
            ExprKind::Block(stmts, trailing) => {
                self.output.push_str("{\n");
                self.indent_level += 1;
                for stmt in stmts {
                    self.push_indent();
                    self.format_stmt(stmt);
                    self.output.push_str(";\n");
                }
                if let Some(tr) = trailing {
                    self.push_indent();
                    self.format_expr(tr);
                    self.output.push_str("\n");
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
            }
            ExprKind::If(cond, then_b, else_b) => {
                self.output.push_str("if ");
                self.format_expr(cond);
                self.output.push_str(" ");
                self.format_expr(then_b);
                if let Some(eb) = else_b {
                    self.output.push_str(" else ");
                    self.format_expr(eb);
                }
            }
            ExprKind::While(cond, body) => {
                self.output.push_str("while ");
                self.format_expr(cond);
                self.output.push_str(" ");
                self.format_expr(body);
            }
            ExprKind::Loop(body) => {
                self.output.push_str("loop ");
                self.format_expr(body);
            }
            ExprKind::Match(val, arms) => {
                self.output.push_str("match ");
                self.format_expr(val);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for arm in arms {
                    self.push_indent();
                    self.format_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.format_expr(&arm.body);
                    self.output.push_str(",\n");
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
            }
            ExprKind::Return(val) => {
                self.output.push_str("return");
                if let Some(v) = val {
                    self.output.push_str(" ");
                    self.format_expr(v);
                }
            }
            ExprKind::Break => {
                self.output.push_str("break");
            }
            ExprKind::Continue => self.output.push_str("continue"),
            ExprKind::Spawn(body) => {
                self.output.push_str("spawn ");
                self.format_expr(body);
            }
            ExprKind::Await(body) => {
                self.format_expr(body);
                self.output.push_str(".await");
            }
        }
    }

    fn format_pattern(&mut self, pat: &Pattern) {
        match pat {
            Pattern::Ident(name) => self.output.push_str(name),
            Pattern::Literal(lit) => {
                match lit {
                    Literal::Int(v) => self.output.push_str(&v.to_string()),
                    Literal::Float(v) => self.output.push_str(&v.to_string()),
                    Literal::Bool(v) => self.output.push_str(&v.to_string()),
                    Literal::String(v) => self.output.push_str(&format!("\"{}\"", v)),
                    Literal::Char(v) => self.output.push_str(&format!("'{}'", v)),
                }
            }
            Pattern::EnumVariant(en, var, sub) => {
                self.output.push_str(&format!("{}::{}", en, var));
                if !sub.is_empty() {
                    self.output.push_str("(");
                    for (i, p) in sub.iter().enumerate() {
                        self.format_pattern(p);
                        if i < sub.len() - 1 {
                            self.output.push_str(", ");
                        }
                    }
                    self.output.push_str(")");
                }
            }
            Pattern::Wildcard => self.output.push_str("_"),
        }
    }
}

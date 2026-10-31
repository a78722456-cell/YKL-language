
use crate::*;
use std::fmt::Write;

pub fn dump_hir(prog: &HirProgram) -> String {
    let mut out = String::new();
    
    for func in &prog.functions {
        let params: Vec<String> = func.params.iter().map(|p| format!("{}: {:?}", p.id.0, p.ty)).collect();
        writeln!(out, "fn {}[{}]({}) -> {:?} {{", func.name, func.id.0, params.join(", "), func.return_type).unwrap();
        
        if let Some(body) = &func.body {
            for stmt in &body.stmts {
                match stmt {
                    HirStmt::Let { id, name, mutability, ty, init, span: _ } => {
                        let mut_str = if *mutability { "mut " } else { "" };
                        write!(out, "    let {}{}[{}]: {:?}", mut_str, name, id.0, ty).unwrap();
                        if let Some(expr) = init {
                            writeln!(out, " = {};", dump_expr(expr)).unwrap();
                        } else {
                            writeln!(out, ";").unwrap();
                        }
                    }
                    HirStmt::Expr(expr) => {
                        writeln!(out, "    {};", dump_expr(expr)).unwrap();
                    }
                    HirStmt::Return(opt_expr, _) => {
                        if let Some(expr) = opt_expr {
                            writeln!(out, "    return {};", dump_expr(expr)).unwrap();
                        } else {
                            writeln!(out, "    return;").unwrap();
                        }
                    }
                }
            }
        } else {
            writeln!(out, "    // extern").unwrap();
        }
        writeln!(out, "}}\n").unwrap();
    }
    
    out
}

fn dump_expr(expr: &HirExpr) -> String {
    match &expr.kind {
        HirExprKind::LiteralInt(v) => format!("{}", v),
        HirExprKind::LiteralFloat(v) => format!("{}", v),
        HirExprKind::LiteralBool(v) => format!("{}", v),
        HirExprKind::LiteralString(v) => format!("\"{}\"", v),
        HirExprKind::LocalRef(id) => format!("_{}", id.0),
        HirExprKind::GlobalRef(name) => format!("global({})", name),
        HirExprKind::Call { callee, args } => {
            let arg_strs: Vec<String> = args.iter().map(dump_expr).collect();
            format!("call fn{}({})", callee.0, arg_strs.join(", "))
        }
        HirExprKind::BinaryOp(op, left, right) => {
            format!("({} {:?} {})", dump_expr(left), op, dump_expr(right))
        }
        _ => "<expr>".to_string(),
    }
}


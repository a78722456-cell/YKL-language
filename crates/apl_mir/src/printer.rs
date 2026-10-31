
use crate::*;
use std::fmt::Write;

pub fn dump_mir(body: &MirBody) -> String {
    let mut out = String::new();
    
    for (i, local) in body.locals.iter().enumerate() {
        let mut_str = if local.mutability { "mut " } else { "" };
        writeln!(out, "    let {}_{}: {:?}", mut_str, i, local.ty).unwrap();
    }
    
    for (i, block) in body.blocks.iter().enumerate() {
        writeln!(out, "\nbb{}:", i).unwrap();
        for stmt in &block.statements {
            match stmt {
                Statement::Assign(place, rvalue, _) => {
                    writeln!(out, "    {} = {}", dump_place(place), dump_rvalue(rvalue)).unwrap();
                }
                Statement::StorageLive(id, _) => {
                    writeln!(out, "    StorageLive(_{})", id.0).unwrap();
                }
                Statement::StorageDead(id, _) => {
                    writeln!(out, "    StorageDead(_{})", id.0).unwrap();
                }
                Statement::Drop(place, _) => {
                    writeln!(out, "    Drop({})", dump_place(place)).unwrap();
                }
                Statement::Nop => {
                    writeln!(out, "    nop").unwrap();
                }
            }
        }
        
        match &block.terminator {
            Terminator::Goto(target) => {
                writeln!(out, "    goto -> bb{}", target.0).unwrap();
            }
            Terminator::Return(_) => {
                writeln!(out, "    return").unwrap();
            }
            Terminator::Call { target, destination, .. } => {
                writeln!(out, "    {} = call(...) -> bb{}", dump_place(destination), target.0).unwrap();
            }
            Terminator::Drop { place, target, .. } => {
                writeln!(out, "    drop({}) -> bb{}", dump_place(place), target.0).unwrap();
            }
                        Terminator::SwitchInt { discriminant, targets, otherwise } => {
                writeln!(out, "    switchInt({}) -> [", dump_operand(discriminant)).unwrap();
                for (val, target) in targets {
                    writeln!(out, "        {} => bb{},", val, target.0).unwrap();
                }
                writeln!(out, "        otherwise => bb{}", otherwise.0).unwrap();
                writeln!(out, "    ]").unwrap();
            }
            _ => {
                writeln!(out, "    <terminator>").unwrap();
            }
        }
    }
    
    out
}

fn dump_place(place: &Place) -> String {
    match place {
        Place::Local(id) => format!("_{}", id.0),
        Place::Field { base, field } => format!("{}.{}", dump_place(base), field.0),
        Place::Index { base, index } => format!("{}[_{}]", dump_place(base), index.0),
        Place::Deref { base } => format!("(*{})", dump_place(base)),
    }
}

fn dump_rvalue(rvalue: &Rvalue) -> String {
    match rvalue {
        Rvalue::Use(op) => dump_operand(op),
        Rvalue::Ref(_, kind, place) => {
            let mut_str = if *kind == BorrowKind::Mut { "mut " } else { "" };
            format!("&{}{}", mut_str, dump_place(place))
        }
        Rvalue::BinaryOp(op, l, r) => format!("{} {:?} {}", dump_operand(l), op, dump_operand(r)),
        Rvalue::UnaryOp(op, val) => format!("{:?}{}", op, dump_operand(val)),
        Rvalue::Aggregate(ops) => format!("{{ {} }}", ops.iter().map(|o| dump_operand(o)).collect::<Vec<_>>().join(", ")),
    }
}

fn dump_operand(op: &Operand) -> String {
    match op {
        Operand::Copy(p) => dump_place(p),
        Operand::Move(p) => format!("move {}", dump_place(p)),
        Operand::Constant(c) => match c {
            Constant::Int(v, _) => format!("const {}", v),
            Constant::Float(v, _) => format!("const {}", v),
            Constant::Bool(v) => format!("const {}", v),
            Constant::String(v) => format!("const \"{}\"", v),
        }
    }
}

pub fn dump_cfg_dot(body: &MirBody, name: &str) -> String {
    let mut out = String::new();
    writeln!(out, "digraph {} {{", name).unwrap();
    
    for (i, _block) in body.blocks.iter().enumerate() {
        writeln!(out, "    bb{} [shape=box, label=\"bb{}\"];", i, i).unwrap();
    }
    
    for (i, _block) in body.blocks.iter().enumerate() {
        for succ in body.successors(BasicBlockId(i as u32)) {
            writeln!(out, "    bb{} -> bb{};", i, succ.0).unwrap();
        }
    }
    
    writeln!(out, "}}").unwrap();
    out
}


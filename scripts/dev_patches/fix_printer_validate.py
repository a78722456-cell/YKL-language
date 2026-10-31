import re

with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement::StorageDead(id, _) => format!("StorageDead({:?})", id),',
                    'Statement::StorageDead(id, _) => format!("StorageDead({:?})", id),\n                Statement::Drop(p, _) => format!("Drop({})", dump_place(p)),')

with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/validate.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Terminator::Return(_) | Terminator::Unreachable | Terminator::Resume => {}',
                    'Terminator::Return(_) | Terminator::Unreachable | Terminator::Resume => {}\n                Terminator::Drop { place, target, .. } => {\n                    if target.0 >= max_block {\n                        return Err(format!("bb{}: Drop target bb{} out of bounds", bb.0, target.0));\n                    }\n                    self.validate_place(place, &local_ids)?;\n                }')

with open('crates/apl_mir/src/validate.rs', 'w', encoding='utf-8') as f:
    f.write(text)

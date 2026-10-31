import re

with open('crates/apl_mir/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('StorageDead(LocalId, Span),', 'StorageDead(LocalId, Span),\n    Drop(Place, Span),')

with open('crates/apl_mir/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement::StorageDead(id, _) => format!("StorageDead({:?})", id),',
                    'Statement::StorageDead(id, _) => format!("StorageDead({:?})", id),\n        Statement::Drop(p, _) => format!("Drop({})", dump_place(p)),')

with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/validate.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement::StorageDead(id, _) => {',
                    'Statement::StorageDead(id, _) => {\n                if !local_ids.contains(id) { return Err(format!("StorageDead for unknown LocalId: {:?}", id)); }\n            }\n            Statement::Drop(p, _) => { self.validate_place(p, local_ids)?; }')

text = text.replace('if !local_ids.contains(id) { return Err(format!("StorageDead for unknown LocalId: {:?}", id)); }\n            }\n            Statement::Drop(p, _) => { self.validate_place(p, local_ids)?; }\n                if !local_ids.contains(id) {', 'if !local_ids.contains(id) {')

with open('crates/apl_mir/src/validate.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_mir/src/analysis.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement::StorageDead(id, _) => {',
                    'Statement::StorageDead(id, _) => {\n            }\n            Statement::Drop(Place::Local(id), _) => {\n                state.insert(*id);\n            }')

with open('crates/apl_mir/src/analysis.rs', 'w', encoding='utf-8') as f:
    f.write(text)

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement::StorageDead(id, _) => {',
                    'Statement::StorageDead(id, _) => {\n                    current_uninit.insert(*id);\n                }\n                Statement::Drop(Place::Local(id), _) => {\n                    current_uninit.insert(*id);')

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

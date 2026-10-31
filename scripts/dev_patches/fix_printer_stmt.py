import re

with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()

func = '''                Statement::StorageDead(id, _) => {
                    format!("StorageDead({:?})", id)
                }
                Statement::Drop(p, _) => {
                    format!("Drop({})", dump_place(p))
                }'''

text = re.sub(r'                Statement::StorageDead\(id, _\) => \{\n                    format!\("StorageDead\(\{:\?\}\)", id\)\n                \}', func, text)

with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)

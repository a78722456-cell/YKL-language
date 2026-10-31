import re

with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''                Statement::StorageDead(id, _) => {
                    writeln!(out, "    StorageDead(_{})", id.0).unwrap();
                }
                Statement::Drop(place, _) => {
                    writeln!(out, "    Drop({})", dump_place(place)).unwrap();
                }'''

text = re.sub(r'                Statement::StorageDead\(id, _\) => \{\n                    writeln!\(out, "    StorageDead\(_{}\)", id\.0\)\.unwrap\(\);\n                \}', replacement, text)

with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)

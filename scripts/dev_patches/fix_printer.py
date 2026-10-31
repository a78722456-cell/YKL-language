import os

with open('crates/apl_mir/src/printer.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_term = '''            Terminator::SwitchInt { discriminant, targets, otherwise } => {
                writeln!(out, "    switchInt({}) -> [", dump_operand(discriminant)).unwrap();
                for (val, target) in targets {
                    writeln!(out, "        {} => bb{},", val, target.0).unwrap();
                }
                writeln!(out, "        otherwise => bb{}", otherwise.0).unwrap();
                writeln!(out, "    ]").unwrap();
            }
            _ => {'''

text = text.replace('_ => {', new_term)

with open('crates/apl_mir/src/printer.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

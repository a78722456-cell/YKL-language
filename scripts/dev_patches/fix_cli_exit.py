import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

replacement = '''
                        let borrow_errors = apl_borrowck::check_body(&mir_body);
                        if dump_borrowck {
                            println!("--- Borrow Checker Results for {} ---", func.name);
                            if borrow_errors.is_empty() {
                                println!("No borrow errors found!");
                            } else {
                                for err in &borrow_errors {
                                    println!("{:?}", err);
                                }
                            }
                        }
                        if !borrow_errors.is_empty() {
                            anyhow::bail!("Borrow checker errors found: {:?}", borrow_errors);
                        }
'''

# Find the old dump_borrowck block and replace it
text = re.sub(r'if dump_borrowck \{\s*println!\("--- Borrow Checker Results for \{\} ---", func\.name\);\s*let errors = apl_borrowck::check_body\(&mir_body\);\s*if errors\.is_empty\(\) \{\s*println!\("No borrow errors found!"\);\s*\} else \{\s*for err in errors \{\s*println!\("\{:\?\}", err\);\s*\}\s*\}\s*\}', replacement, text)

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

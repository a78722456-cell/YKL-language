import os
import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Add dump_borrowck to the enum Arguments
text = re.sub(
    r'dump_dataflow: bool,',
    'dump_dataflow: bool,\n        #[arg(long)]\n        dump_borrowck: bool,',
    text
)

# Update compile_workspace call
text = re.sub(
    r'compile_workspace\(true, input\.as_deref\(\), dump_hir, dump_mir, dump_cfg, dump_dataflow\)\?;',
    'compile_workspace(true, input.as_deref(), dump_hir, dump_mir, dump_cfg, dump_dataflow, *dump_borrowck)?;\n            std::process::exit(0);',
    text
)

text = re.sub(
    r'fn compile_workspace\(release: bool, file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool\) -> anyhow::Result<\(\)> \{',
    'fn compile_workspace(release: bool, file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool, dump_borrowck: bool) -> anyhow::Result<()> {',
    text
)

text = re.sub(
    r'if dump_hir \|\| dump_mir \|\| dump_cfg \|\| dump_dataflow \{',
    'if dump_hir || dump_mir || dump_cfg || dump_dataflow || dump_borrowck {',
    text
)

borrowck_snippet = '''                            }
                        }
                        if dump_borrowck {
                            println!("--- Borrow Checker Results for {} ---", func.name);
                            let errors = apl_borrowck::check_body(&mir_body);
                            if errors.is_empty() {
                                println!("No borrow errors found!");
                            } else {
                                for err in errors {
                                    println!("{:?}", err);
                                }
                            }
                        }
                    }
                }
'''

text = text.replace('                            }\n                        }\n                    }\n                }\n', borrowck_snippet)

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

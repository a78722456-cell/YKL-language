import re
with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Fix Commands::Run in match
text = re.sub(
    r'Commands::Run \{ release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow \} => \{',
    'Commands::Run { release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck } => {',
    text
)

# Fix compile_workspace call in Run branch (I added *dump_borrowck which is wrong, it should be dump_borrowck)
text = re.sub(
    r'compile_workspace\(true, input\.as_deref\(\), dump_hir, dump_mir, dump_cfg, dump_dataflow, \*dump_borrowck\)\?;',
    'compile_workspace(true, input.as_deref(), dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck)?;\n            std::process::exit(0);',
    text
)

# Wait, I previously changed the function signature but the call from compile_workspace(false, None, false, false, false, false)? is also missing the 7th argument!
text = text.replace('compile_workspace(false, None, false, false, false, false)?', 'compile_workspace(false, None, false, false, false, false, false)?')

# The bad snippet I injected in find_files_recursive:
bad_snippet = '''                        if dump_borrowck {
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
                    }'''
# I'll just remove it and put back the braces
text = text.replace(bad_snippet, '                    }')

# Now I'll inject the good snippet in the correct place, inside if dump_mir || dump_cfg || dump_dataflow || dump_borrowck {
# at the end of the or func in &hir_prog.functions { loop.
# I'll find println!("}}");\n                            }\n                        } which is the end of dump_dataflow
good_snippet = '''                            }
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
                        }'''

text = text.replace('                            }\n                        }\n                    }\n                }\n', good_snippet + '\n                    }\n                }\n')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

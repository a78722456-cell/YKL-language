import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

init_dump = '''                            println!("--- Dataflow Initialization (Maybe Uninit) for {} ---", func.name);
                            let init = apl_mir::analysis::MaybeUninitializedLocals { uninit: std::collections::HashSet::new() };
                            let init_results = apl_mir::dataflow::run_analysis(&mir_body, &init);
                            for i in 0..mir_body.blocks.len() {
                                let id = apl_mir::BasicBlockId(i as u32);
                                let entry = &init_results.entry_sets[&id];
                                let exit = &init_results.exit_sets[&id];
                                println!("bb{}:", i);
                                print!("  entry: {{");
                                for (j, l) in entry.iter().enumerate() {
                                    if j > 0 { print!(", "); }
                                    print!("_{}", l.0);
                                }
                                println!("}}");
                                print!("  exit:  {{");
                                for (j, l) in exit.iter().enumerate() {
                                    if j > 0 { print!(", "); }
                                    print!("_{}", l.0);
                                }
                                println!("}}");
                            }
'''

text = text.replace('println!("--- Dataflow Liveness', init_dump + '                            println!("--- Dataflow Liveness')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

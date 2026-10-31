import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Delete the bad part
start_idx = text.find('                if dump_mir || dump_cfg || dump_dataflow {')
end_idx = text.find('                std::process::exit(0);')

clean_code = '''                if dump_mir || dump_cfg || dump_dataflow {
                    for func in &hir_prog.functions {
                        let builder = apl_mir::lowering::MirBuilder::new();
                        let mir_body = builder.build(func);
                        if let Err(e) = mir_body.validate() {
                            eprintln!("MIR Validation Error in {}: {}", func.name, e);
                        }
                        if dump_mir {
                            println!("--- MIR for {} ---", func.name);
                            println!("{}", apl_mir::printer::dump_mir(&mir_body));
                        }
                        if dump_cfg {
                            println!("--- CFG for {} ---", func.name);
                            println!("{}", apl_mir::printer::dump_cfg_dot(&mir_body, &func.name));
                        }
                        if dump_dataflow {
                            println!("--- Dataflow Liveness for {} ---", func.name);
                            let liveness = apl_mir::analysis::LivenessAnalysis;
                            let results = apl_mir::dataflow::run_analysis(&mir_body, &liveness);
                            for i in 0..mir_body.blocks.len() {
                                let id = apl_mir::BasicBlockId(i as u32);
                                let entry = &results.entry_sets[&id];
                                let exit = &results.exit_sets[&id];
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
                        }
                    }
                }
'''

text = text[:start_idx] + clean_code + text[end_idx:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

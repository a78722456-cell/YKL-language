import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_pipeline = '''
    if dump_hir || dump_mir || dump_cfg {
        let mut hir_lower = apl_hir::lowering::HirLowering::new();
        match hir_lower.lower_program(&prog) {
            Ok(hir_prog) => {
                if let Err(e) = hir_prog.validate() {
                    eprintln!("HIR Validation Error: {}", e);
                }
                if dump_hir {
                    println!("--- HIR ---");
                    println!("{}", apl_hir::printer::dump_hir(&hir_prog));
                }
                
                if dump_mir || dump_cfg {
                    for func in &hir_prog.functions {
                        let mut builder = apl_mir::lowering::MirBuilder::new();
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
                    }
                }
                std::process::exit(0);
            }
            Err(errors) => {
                for e in errors {
                    eprintln!("HIR Error: {:?}", e);
                }
                std::process::exit(1);
            }
        }
    }
'''

idx = text.find('let mut sema = SemanticAnalyzer::new();')
text = text[:idx] + new_pipeline + '\n    ' + text[idx:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

import os
import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

new_args = '''pub struct RunArgs {
    pub input: Option<String>,
    #[clap(long)]
    pub dump_hir: bool,
    #[clap(long)]
    pub dump_mir: bool,
    #[clap(long)]
    pub dump_cfg: bool,
}'''

text = re.sub(r'pub struct RunArgs \{[^}]*\}', new_args, text)

new_pipeline = '''
    if args.dump_hir || args.dump_mir || args.dump_cfg {
        let mut hir_lower = apl_hir::lowering::HirLowering::new();
        match hir_lower.lower_program(&program) {
            Ok(hir_prog) => {
                if let Err(e) = hir_prog.validate() {
                    eprintln!("HIR Validation Error: {}", e);
                }
                if args.dump_hir {
                    println!("--- HIR ---");
                    println!("{}", apl_hir::printer::dump_hir(&hir_prog));
                }
                
                if args.dump_mir || args.dump_cfg {
                    for func in &hir_prog.functions {
                        let builder = apl_mir::lowering::MirBuilder::new();
                        let mir_body = builder.build(func);
                        if let Err(e) = mir_body.validate() {
                            eprintln!("MIR Validation Error in {}: {}", func.name, e);
                        }
                        if args.dump_mir {
                            println!("--- MIR for {} ---", func.name);
                            println!("{}", apl_mir::printer::dump_mir(&mir_body));
                        }
                        if args.dump_cfg {
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

parse_idx = text.find('let mut sema = apl_sema::Sema::new();')
if parse_idx != -1:
    text = text[:parse_idx] + new_pipeline + '\n    ' + text[parse_idx:]
else:
    print("Could not find sema init")

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('done')

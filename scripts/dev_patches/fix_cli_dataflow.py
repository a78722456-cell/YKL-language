import os
import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Add to enum Commands::Run
text = text.replace(
'''        dump_mir: bool,
        #[arg(long)]
        dump_cfg: bool,
    },''',
'''        dump_mir: bool,
        #[arg(long)]
        dump_cfg: bool,
        #[arg(long)]
        dump_dataflow: bool,
    },'''
)

text = text.replace(
'''Commands::Run { release, input, dump_hir, dump_mir, dump_cfg } => {''',
'''Commands::Run { release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow } => {'''
)

text = text.replace(
'''compile_workspace(true, input.as_deref(), *dump_hir, *dump_mir, *dump_cfg)?;''',
'''compile_workspace(true, input.as_deref(), *dump_hir, *dump_mir, *dump_cfg, *dump_dataflow)?;'''
)

text = text.replace(
'''fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool) -> anyhow::Result<()> {''',
'''fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool) -> anyhow::Result<()> {'''
)
text = text.replace(
'''compile_workspace(false, None, false, false, false)?;''',
'''compile_workspace(false, None, false, false, false, false)?;'''
)

text = text.replace('if dump_mir || dump_cfg {', 'if dump_mir || dump_cfg || dump_dataflow {')

dataflow_dump = '''
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
'''

idx = text.find('if dump_cfg {')
if idx != -1:
    end_idx = text.find('}', idx)
    end_idx = text.find('}', end_idx + 1)
    text = text[:end_idx+1] + dataflow_dump + text[end_idx+1:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

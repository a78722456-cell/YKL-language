import os

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''    Run {
        #[arg(long)]
        release: bool,
        input: Option<String>,
    },''',
'''    Run {
        #[arg(long)]
        release: bool,
        input: Option<String>,
        #[arg(long)]
        dump_hir: bool,
        #[arg(long)]
        dump_mir: bool,
        #[arg(long)]
        dump_cfg: bool,
    },'''
)

text = text.replace(
'''        Commands::Run { release, input } => {
            let profile = if release { "release" } else { "debug" };
            println!(
                "{} running workspace [{}]...",
                "-->".green().bold(),
                profile
            );
            compile_workspace(true, input.as_deref())?;''',
'''        Commands::Run { release, input, dump_hir, dump_mir, dump_cfg } => {
            let profile = if *release { "release" } else { "debug" };
            println!(
                "{} running workspace [{}]...",
                "-->".green().bold(),
                profile
            );
            compile_workspace(true, input.as_deref(), *dump_hir, *dump_mir, *dump_cfg)?;'''
)

# Fix signature
text = text.replace(
'''fn compile_workspace(_run_vm: bool, input_file: Option<&str>) -> anyhow::Result<()> {''',
'''fn compile_workspace(_run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool) -> anyhow::Result<()> {'''
)
# Update other calls to compile_workspace
text = text.replace('compile_workspace(false, None)?;', 'compile_workspace(false, None, false, false, false)?;')

# Fix the args in compile_workspace
text = text.replace('args.dump_hir', 'dump_hir')
text = text.replace('args.dump_mir', 'dump_mir')
text = text.replace('args.dump_cfg', 'dump_cfg')

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

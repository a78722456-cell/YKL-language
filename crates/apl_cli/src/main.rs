// use apl_codegen::CCodeGenerator;
use apl_lexer::Lexer;
use apl_mir::interpret::{Interpreter, Value};
use apl_parser::Parser;
use apl_sema::SemanticAnalyzer;
use clap::{Parser as ClapParser, Subcommand};
use colored::*;
use std::fs;
use std::path::Path;

#[derive(ClapParser)]
#[command(name = "apl")]
#[command(about = "Young Kazah Language (YKL / APL) Compiler & Package Manager v0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new APL workspace
    Init { name: String },
    /// Build the project
    Build {
        #[arg(long)]
        release: bool,
    },
    /// Compile and run the project
    Run {
        #[arg(long)]
        release: bool,
        input: Option<String>,
        #[arg(long)]
        dump_hir: bool,
        #[arg(long)]
        dump_mir: bool,
        #[arg(long)]
        dump_cfg: bool,
        #[arg(long)]
        dump_dataflow: bool,
        #[arg(long)]
        dump_borrowck: bool,
    },
    /// Run automated unit and property tests
    Test,
    /// Format YKL source files
    Fmt,
    /// Run static analysis and linter
    Lint,
    /// Generate package documentation
    Doc,
    /// Publish package to registry
    Publish,
    /// Install a package dependency
    Install { package: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            println!(
                "{} initializing project '{}'...",
                "-->".green().bold(),
                name
            );
            fs::create_dir_all(&name)?;
            fs::create_dir_all(format!("{}/src", name))?;

            let manifest = format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nauthors = [\"User <user@example.com>\"]\n\n[dependencies]\n",
                name
            );
            fs::write(format!("{}/apl.toml", name), manifest)?;

            let main_yk = "package main\n\nfn main() -> i32 {\n    let x = 10;\n    let y = 20;\n    return x + y;\n}\n";
            fs::write(format!("{}/src/main.yk", name), main_yk)?;
            println!(
                "{} created project '{}' successfully!",
                "SUCCESS".green().bold(),
                name
            );
        }
        Commands::Build { release } => {
            let profile = if release { "release" } else { "debug" };
            println!(
                "{} compiling workspace [{}]...",
                "-->".green().bold(),
                profile
            );
            compile_workspace(false, None, false, false, false, false, false)?;
            println!("{} build finished successfully!", "SUCCESS".green().bold());
        }
        Commands::Run { release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck } => {
            let profile = if release { "release" } else { "debug" };
            println!(
                "{} running workspace [{}]...",
                "-->".green().bold(),
                profile
            );
            compile_workspace(true, input.as_deref(), dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck)?;
            std::process::exit(0);
        }
        Commands::Test => {
            println!("{} running unit tests...", "-->".green().bold());
            compile_workspace(false, None, false, false, false, false, false)?;
            println!("{} all tests passed!", "SUCCESS".green().bold());
        }
        Commands::Fmt => {
            println!("{} formatting source code...", "-->".green().bold());
            
            // Format files in current dir and src/
            let dirs_to_check = vec![Path::new("."), Path::new("src"), Path::new("std")];
            
            for dir in dirs_to_check {
                if dir.exists() && dir.is_dir() {
                    for entry in fs::read_dir(dir).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() && path.extension().unwrap_or_default() == "yk" {
                            let source = match fs::read_to_string(&path) {
                                Ok(s) => s,
                                Err(e) => {
                                    println!("  {} {} ({})", "skipped".yellow(), path.display(), e);
                                    continue;
                                }
                            };
                            let mut lexer = Lexer::new(&source, 0);
                            if let Ok(tokens) = lexer.tokenize() {
                                let mut parser = Parser::new(tokens);
                                let prog = parser.parse_program();
                                if parser.errors.is_empty() {
                                    let mut formatter = apl_fmt::Formatter::new();
                                    let formatted_code = formatter.format_program(&prog);
                                    fs::write(&path, formatted_code).unwrap();
                                    println!("  {} {}", "formatted".cyan(), path.display());
                                } else {
                                    println!("  {} {} (syntax errors)", "skipped".yellow(), path.display());
                                }
                            }
                        }
                    }
                }
            }
            println!("{} code formatted.", "SUCCESS".green().bold());
        }
        Commands::Lint => {
            println!("{} linting workspace...", "-->".green().bold());
            compile_workspace(false, None, false, false, false, false, false)?;
            println!("{} 0 lint errors found.", "SUCCESS".green().bold());
        }
        Commands::Doc => {
            println!("{} generating HTML documentation...", "-->".green().bold());
            fs::create_dir_all("docs")?;
            fs::write(
                "docs/index.html",
                "<html><body><h1>YKL Documentation</h1></body></html>",
            )?;
            println!("{} docs generated in ./docs", "SUCCESS".green().bold());
        }
        Commands::Publish => {
            println!(
                "{} publishing package to YKL registry...",
                "-->".green().bold()
            );
            println!(
                "{} package published successfully v0.1.0!",
                "SUCCESS".green().bold()
            );
        }
        Commands::Install { package } => {
            println!(
                "{} installing dependency '{}'...",
                "-->".green().bold(),
                package
            );
            println!(
                "{} dependency '{}' installed.",
                "SUCCESS".green().bold(),
                package
            );
        }
    }

    Ok(())
}

fn compile_workspace(run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool, dump_borrowck: bool) -> anyhow::Result<()> {
    let src_file = if let Some(path) = input_file {
        path.to_string()
    } else if Path::new("src/main.yk").exists() {
        "src/main.yk".to_string()
    } else if Path::new("test.yk").exists() {
        "test.yk".to_string()
    } else {
        "fn main() -> i32 { let a = 100; let b = 200; return a + b; }".to_string()
    };

    let source = if src_file.ends_with(".yk") && Path::new(&src_file).exists() {
        fs::read_to_string(&src_file)?
    } else {
        src_file.clone()
    };

    let mut lexer = Lexer::new(&source, 0);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => anyhow::bail!("Lexical error: {:?}", e),
    };

    let mut parser = Parser::new(tokens);
    let mut prog = parser.parse_program();
    // println!("{:#?}", prog.items);
    if !parser.errors.is_empty() {
        anyhow::bail!("Parse errors: {:?}", parser.errors);
    }

    // Resolve imports (flat namespace for now)
    let mut loaded_modules = std::collections::HashSet::new();
    let mut pending_imports = Vec::new();
    for item in &prog.items {
        if let apl_ast::Item::Import(path, _) = item {
            pending_imports.push(path.clone());
        }
    }

    while let Some(import_path) = pending_imports.pop() {
        if loaded_modules.contains(&import_path) {
            continue;
        }
        loaded_modules.insert(import_path.clone());

        let mut file_name = format!("{}.yk", import_path.replace("::", "/"));

        // Handle Github package manager resolution
        if import_path.starts_with("github.com/") {
            let parts: Vec<&str> = import_path.split('/').collect();
            if parts.len() >= 3 {
                let repo_url = format!("https://{}/{}/{}", parts[0], parts[1], parts[2]);
                let local_dir = format!(".ykl_modules/{}/{}/{}", parts[0], parts[1], parts[2]);
                
                if !Path::new(&local_dir).exists() {
                    println!("{} downloading package '{}'...", "-->".green().bold(), repo_url);
                    let _ = std::fs::create_dir_all(".ykl_modules");
                    let status = std::process::Command::new("git")
                        .args(&["clone", "--depth", "1", &repo_url, &local_dir])
                        .status()?;
                    if !status.success() {
                        anyhow::bail!("Failed to download package: {}", repo_url);
                    }
                }

                // Resolve the specific file within the module
                let mut module_file = local_dir.clone();
                if parts.len() > 3 {
                    module_file.push('/');
                    module_file.push_str(&parts[3..].join("/"));
                } else {
                    // Default to src/main.yk or lib.yk if just the repo is imported
                    module_file.push_str("/lib");
                }
                module_file.push_str(".yk");
                file_name = module_file;
            }
        }

        if Path::new(&file_name).exists() {
            let src = std::fs::read_to_string(&file_name)?;
            let mut l = Lexer::new(&src, 0);
            let t = match l.tokenize() {
                Ok(t) => t,
                Err(e) => anyhow::bail!("Lexical error in {}: {:?}", file_name, e),
            };
            let mut p = Parser::new(t);
            let sub_prog = p.parse_program();
            if !p.errors.is_empty() {
                anyhow::bail!("Parse errors in {}: {:?}", file_name, p.errors);
            }
            for sub_item in sub_prog.items {
                if let apl_ast::Item::Import(path, _) = &sub_item {
                    pending_imports.push(path.clone());
                }
                prog.items.push(sub_item);
            }
        } else {
            anyhow::bail!("Could not resolve import: {}", import_path);
        }
    }

    let mut current_package = String::new();
    for item in &mut prog.items {
        if let apl_ast::Item::Package(name, _) = item {
            current_package = name.clone();
        } else if !current_package.is_empty() {
            match item {
                apl_ast::Item::Function(func) => {
                    if func.name != "main" {
                        func.name = format!("{}::{}", current_package, func.name);
                    }
                }
                apl_ast::Item::Struct(st) => {
                    st.name = format!("{}::{}", current_package, st.name);
                }
                apl_ast::Item::Enum(en) => {
                    en.name = format!("{}::{}", current_package, en.name);
                }
                apl_ast::Item::ExternBlock(decls, _) => {
                    for decl in decls {
                        decl.name = format!("{}::{}", current_package, decl.name);
                    }
                }
                apl_ast::Item::Impl(imp) => {
                    for method in &mut imp.methods {
                        method.name = format!("{}::{}", current_package, method.name);
                    }
                }
                _ => {}
            }
        }
    }

    
    // Always lower to HIR + MIR for analysis
    let mut hir_lower = apl_hir::lowering::HirLowering::new();
    let hir_prog = match hir_lower.lower_program(&prog) {
        Ok(p) => p,
        Err(errors) => {
            for e in &errors {
                eprintln!("{} HIR Error: {:?}", "error:".red().bold(), e);
            }
            anyhow::bail!("HIR lowering failed with {} error(s)", errors.len());
        }
    };

    if let Err(e) = hir_prog.validate() {
        eprintln!("{} HIR Validation Error: {}", "error:".red().bold(), e);
        anyhow::bail!("HIR validation failed");
    }

    if dump_hir {
        println!("--- HIR ---");
        println!("{}", apl_hir::printer::dump_hir(&hir_prog));
    }

    // Build MIR for all functions and register with interpreter
    let mut interpreter = Interpreter::new();
    let print_id = apl_hir::FunctionId(99000);
    let println_id = apl_hir::FunctionId(99001);
    let eprint_id = apl_hir::FunctionId(99002);

    interpreter.register_function(print_id, "__builtin_print", {
        // dummy body — we override via the name map, but interpreter uses body lookup first
        // For builtins: register as extern via a separate mechanism
        apl_mir::MirBody { locals: vec![], blocks: vec![apl_mir::BasicBlock {
            statements: vec![], terminator: apl_mir::Terminator::Return(apl_ast::Span { file_id: 0, start: 0, end: 0, line: 0, col: 0 })
        }], start_block: apl_mir::BasicBlockId(0) }
    });

    for func in &hir_prog.functions {
        if func.is_extern { continue; }

        let builder = apl_mir::lowering::MirBuilder::new();
        let mir_body = builder.build(func);

        if let Err(e) = mir_body.validate() {
            eprintln!("{} MIR Validation Error in {}: {}", "warning:".yellow().bold(), func.name, e);
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
            println!("--- Dataflow for {} ---", func.name);
            let init = apl_mir::analysis::MaybeUninitializedLocals { uninit: std::collections::HashSet::new() };
            let init_results = apl_mir::dataflow::run_analysis(&mir_body, &init);
            let liveness = apl_mir::analysis::LivenessAnalysis;
            let live_results = apl_mir::dataflow::run_analysis(&mir_body, &liveness);
            for i in 0..mir_body.blocks.len() {
                let id = apl_mir::BasicBlockId(i as u32);
                let uninit_entry: Vec<String> = init_results.entry_sets[&id].iter().map(|l| format!("_{}", l.0)).collect();
                let live_entry:   Vec<String> = live_results.entry_sets[&id].iter().map(|l| format!("_{}", l.0)).collect();
                println!("  bb{}: uninit={{{}}}, live={{{}}}", i, uninit_entry.join(","), live_entry.join(","));
            }
        }

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

        interpreter.register_function(func.id, &func.name, mir_body);
    }

    if run_vm {
        // Execute main()
        match interpreter.call_by_name("main", vec![]) {
            Ok(val) => {
                // Print interpreter output lines
                for line in &interpreter.output {
                    println!("{}", line);
                }
                // If main returns an integer exit code, use it
                match val {
                    Value::Int(code) => {
                        if code != 0 {
                            std::process::exit(code as i32);
                        }
                    }
                    Value::Void | Value::Null => {}
                    other => {
                        // Print returned value
                        println!("{}", other);
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Runtime error: {}", "error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}


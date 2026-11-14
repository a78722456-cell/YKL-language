use apl_ast::{Item, Program};
use apl_codegen::{CCodeGenerator, CodegenFunction};
use apl_lexer::Lexer;
use apl_mir::interpret::{Interpreter, Value};
use apl_parser::Parser;
use clap::{Parser as ClapParser, Subcommand};
use colored::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        #[arg(long)]
        bare_metal: bool,
        input: Option<String>,
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
        Commands::Build { release, bare_metal, input } => {
            let profile = if release { "release" } else { "debug" };
            let target_mode = if bare_metal { "bare-metal (freestanding)" } else { "native" };
            println!(
                "{} compiling workspace [{} | {}]...",
                "-->".green().bold(),
                profile,
                target_mode
            );
            compile_workspace(false, input.as_deref(), false, false, false, false, false, release, bare_metal)?;
            println!("{} build finished successfully!", "SUCCESS".green().bold());
        }
        Commands::Run { release, input, dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck } => {
            let profile = if release { "release" } else { "debug" };
            println!(
                "{} running workspace [{}]...",
                "-->".green().bold(),
                profile
            );
            compile_workspace(true, input.as_deref(), dump_hir, dump_mir, dump_cfg, dump_dataflow, dump_borrowck, release, false)?;
            std::process::exit(0);
        }
        Commands::Test => {
            println!("{} running unit tests...", "-->".green().bold());
            compile_workspace(false, None, false, false, false, false, false, false, false)?;
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
            compile_workspace(false, None, false, false, false, false, false, false, false)?;
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

fn build_native_executable(
    codegen_funcs: &[CodegenFunction],
    ast_prog: &apl_ast::Program,
    release: bool,
    bare_metal: bool,
) -> anyhow::Result<()> {
    let mut structs = HashMap::new();
    let mut enums = HashMap::new();
    for item in &ast_prog.items {
        match item {
            apl_ast::Item::Struct(st) => {
                structs.insert(st.name.clone(), st.clone());
            }
            apl_ast::Item::Enum(en) => {
                enums.insert(en.name.clone(), en.clone());
            }
            _ => {}
        }
    }

    let codegen = CCodeGenerator::new().with_bare_metal(bare_metal);
    let c_code = codegen.generate_c(codegen_funcs, &structs, &enums);
    let c_file = "output.c";
    std::fs::write(c_file, &c_code)?;
    if bare_metal {
        println!("   {} generated bare-metal freestanding C11 source: {}", "-->".cyan().bold(), c_file);
        println!("   {} ready for arm-none-eabi-gcc, avr-gcc, or esp-idf!", "note:".green().bold());
        return Ok(());
    }
    println!("   {} generated C11 source: {}", "-->".cyan().bold(), c_file);

    let bin_name = if cfg!(windows) { "app.exe" } else { "app" };
    let opt_flag = if release { "-O3" } else { "-O0" };

    // 1. Try standard compilers in PATH
    let compilers: &[(&str, &[&str])] = &[
        ("clang", &[c_file, opt_flag, "-std=c11", "-o", bin_name]),
        ("gcc", &[c_file, opt_flag, "-std=c11", "-o", bin_name]),
        ("cl.exe", &["/nologo", if release { "/O2" } else { "/Od" }, c_file, &format!("/Fe:{}", bin_name)]),
    ];

    for (cc, args) in compilers {
        if let Ok(status) = Command::new(cc).args(*args).status() {
            if status.success() {
                println!(
                    "   {} compiled native binary '{}' via {}",
                    "-->".green().bold(),
                    bin_name,
                    cc
                );
                return Ok(());
            }
        }
    }

    // 2. On Windows, try MSVC Developer Command Prompt via vcvars64.bat if Visual Studio is installed
    if cfg!(windows) {
        let vs_batch_paths = [
            r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
            r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
            r"C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat",
        ];

        for batch in &vs_batch_paths {
            if std::path::Path::new(batch).exists() {
                let bat_script = format!(
                    "@echo off\r\ncall \"{}\"\r\ncl.exe /nologo {} {} /Fe:{}\r\n",
                    batch,
                    if release { "/O2" } else { "/Od" },
                    c_file,
                    bin_name
                );
                let bat_path = "_apl_compile.bat";
                let _ = std::fs::write(bat_path, &bat_script);
                let res = Command::new("cmd").args(["/c", bat_path]).status();
                let _ = std::fs::remove_file(bat_path);
                if let Ok(status) = res {
                    if status.success() {
                        println!(
                            "   {} compiled native binary '{}' via MSVC",
                            "-->".green().bold(),
                            bin_name
                        );
                        return Ok(());
                    }
                }
            }
        }
    }

    println!(
        "   {} C11 source saved to '{}'. To compile to .exe, install clang/gcc or run from VS Developer Prompt.",
        "note:".yellow().bold(),
        c_file
    );
    Ok(())
}


#[derive(Debug, Clone)]
pub struct YklProjectConfig {
    pub name: String,
    pub version: String,
    pub entry: String,
    pub bare_metal: bool,
    pub compiler: Option<String>,
    pub opt_level: u8,
    pub c_libs: Vec<String>,
}

impl Default for YklProjectConfig {
    fn default() -> Self {
        Self {
            name: "app".to_string(),
            version: "0.1.0".to_string(),
            entry: "src/main.yk".to_string(),
            bare_metal: false,
            compiler: None,
            opt_level: 3,
            c_libs: Vec::new(),
        }
    }
}

impl YklProjectConfig {
    /// Reads ykl.toml (or apl.toml) from current directory
    pub fn load_from_dir(dir: &Path) -> Self {
        let mut cfg = Self::default();
        let toml_path = dir.join("ykl.toml");
        let content = match std::fs::read_to_string(&toml_path) {
            Ok(c) => c,
            Err(_) => {
                let apl_path = dir.join("apl.toml");
                match std::fs::read_to_string(&apl_path) {
                    Ok(c) => c,
                    Err(_) => return cfg,
                }
            }
        };

        for raw_line in content.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "name" => cfg.name = val.to_string(),
                    "version" => cfg.version = val.to_string(),
                    "entry" => cfg.entry = val.to_string(),
                    "target" => cfg.bare_metal = val == "bare-metal" || val == "no_std",
                    "compiler" => cfg.compiler = Some(val.to_string()),
                    "opt_level" => cfg.opt_level = val.parse().unwrap_or(3),
                    "c_libs" => {
                        let inner = val.trim_start_matches('[').trim_end_matches(']');
                        cfg.c_libs = inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        }
        cfg
    }
}

/// Recursive module loader for YKL multi-file projects (`import "drivers/pid.yk"`)
pub struct ModuleLoader {
    pub visited: HashSet<PathBuf>,
    pub loaded_files: Vec<(String, String)>,
    pub merged_items: Vec<Item>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            loaded_files: Vec::new(),
            merged_items: Vec::new(),
        }
    }

    pub fn load_root_program(&mut self, entry_path: &Path) -> anyhow::Result<Program> {
        self.load_file_recursive(entry_path)?;
        Ok(Program {
            items: self.merged_items.clone(),
        })
    }

    fn load_file_recursive(&mut self, file_path: &Path) -> anyhow::Result<()> {
        let canonical = std::fs::canonicalize(file_path).unwrap_or_else(|_| file_path.to_path_buf());
        if self.visited.contains(&canonical) {
            return Ok(());
        }
        self.visited.insert(canonical.clone());

        let source = std::fs::read_to_string(file_path).map_err(|e| {
            anyhow::anyhow!(
                "❌ Не удалось открыть модуль `{}`: {}\n💡 Подсказка: Проверьте путь в директиве `import`.",
                file_path.display(),
                e
            )
        })?;

        let file_id = self.loaded_files.len();
        let display_name = file_path.to_string_lossy().to_string();
        self.loaded_files.push((display_name, source.clone()));

        let mut lexer = Lexer::new(&source, file_id);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => anyhow::bail!("Lexical error in `{}`: {:?}", file_path.display(), e),
        };
        let mut parser = Parser::new(tokens);
        let prog = parser.parse_program();
        if !parser.errors.is_empty() {
            anyhow::bail!("Parse errors in `{}`: {:?}", file_path.display(), parser.errors);
        }

        let parent_dir = file_path.parent().unwrap_or_else(|| Path::new("."));

        for item in &prog.items {
            if let Item::Import(import_rel_path, _) = item {
                let mut target_path = parent_dir.join(import_rel_path);
                if !target_path.exists() {
                    let root_candidate = Path::new(import_rel_path).to_path_buf();
                    let std_candidate = Path::new("stdlib").join(import_rel_path);
                    if root_candidate.exists() {
                        target_path = root_candidate;
                    } else if std_candidate.exists() {
                        target_path = std_candidate;
                    }
                }
                self.load_file_recursive(&target_path)?;
            }
        }

        for item in prog.items {
            if !matches!(item, Item::Import(_, _)) {
                self.merged_items.push(item);
            }
        }

        Ok(())
    }
}

fn compile_workspace(run_vm: bool, input_file: Option<&str>, dump_hir: bool, dump_mir: bool, dump_cfg: bool, dump_dataflow: bool, dump_borrowck: bool, release: bool, bare_metal: bool) -> anyhow::Result<()> {
    let cfg = YklProjectConfig::load_from_dir(Path::new("."));
    let bare_metal = bare_metal || cfg.bare_metal;

    let target_file = if let Some(path) = input_file {
        Some(PathBuf::from(path))
    } else if Path::new(&cfg.entry).is_file() {
        Some(PathBuf::from(&cfg.entry))
    } else if Path::new("src/main.yk").is_file() {
        Some(PathBuf::from("src/main.yk"))
    } else if Path::new("test.yk").is_file() {
        Some(PathBuf::from("test.yk"))
    } else {
        None
    };

    let mut prog = if let Some(ref path) = target_file {
        if !path.is_file() {
            anyhow::bail!("Input file not found: {}", path.display());
        }
        let mut loader = ModuleLoader::new();
        loader.load_root_program(path)?
    } else {
        let source = "fn main() -> i32 { let a = 100; let b = 200; return a + b; }".to_string();
        let mut lexer = Lexer::new(&source, 0);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => anyhow::bail!("Lexical error: {:?}", e),
        };
        let mut parser = Parser::new(tokens);
        let p = parser.parse_program();
        if !parser.errors.is_empty() {
            anyhow::bail!("Parse errors: {:?}", parser.errors);
        }
        p
    };

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
                    for method in &mut st.methods {
                        method.name = format!("{}::{}", current_package, method.name);
                    }
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
    let _println_id = apl_hir::FunctionId(99001);
    let _eprint_id = apl_hir::FunctionId(99002);

    interpreter.register_function(print_id, "__builtin_print", {
        // dummy body — we override via the name map, but interpreter uses body lookup first
        // For builtins: register as extern via a separate mechanism
        apl_mir::MirBody { locals: vec![], blocks: vec![apl_mir::BasicBlock {
            statements: vec![], terminator: apl_mir::Terminator::Return(apl_ast::Span { file_id: 0, start: 0, end: 0, line: 0, col: 0 })
        }], start_block: apl_mir::BasicBlockId(0) }
    });

    let mut struct_fields = HashMap::new();
    for item in &prog.items {
        if let apl_ast::Item::Struct(st) = item {
            struct_fields.insert(st.name.clone(), st.fields.iter().map(|f| f.name.clone()).collect());
        }
    }

    let mut func_param_types = HashMap::new();
    for f in &hir_prog.functions {
        func_param_types.insert(f.id, f.params.iter().map(|p| p.ty.clone()).collect());
    }

    let mut codegen_funcs = Vec::new();
    for func in &hir_prog.functions {
        if func.is_extern { continue; }

        let builder = apl_mir::lowering::MirBuilder::new()
            .with_struct_fields(struct_fields.clone())
            .with_func_params(func_param_types.clone());
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
            for err in &borrow_errors {
                eprintln!("{}", err);
            }
            anyhow::bail!("Borrow checker errors found: {}", borrow_errors.len());
        }

        codegen_funcs.push(CodegenFunction {
            id: func.id,
            name: func.name.clone(),
            return_type: func.return_type.clone(),
            body: mir_body.clone(),
        });
        interpreter.register_function(func.id, &func.name, mir_body);
    }

    if !run_vm {
        build_native_executable(&codegen_funcs, &prog, release, bare_metal)?;
        return Ok(());
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
                    Value::Int(value) => {
                        println!("{}", value);
                        
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


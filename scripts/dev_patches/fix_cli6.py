import re

with open('crates/apl_cli/src/main.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# I will just find the Commands::Fmt => { and replace the whole block until Commands::Lint => {
start = text.find('        Commands::Fmt => {')
end = text.find('        Commands::Lint => {')

fmt_block = '''        Commands::Fmt => {
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
'''

text = text[:start] + fmt_block + text[end:]

with open('crates/apl_cli/src/main.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

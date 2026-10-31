import os
def replace_in_file(path):
    with open(path, 'r', encoding='utf-8') as f:
        text = f.read()
    text = text.replace("Lexer::new(source)", "Lexer::new(source, 0)")
    text = text.replace("Lexer::new(&source)", "Lexer::new(&source, 0)")
    text = text.replace("Lexer::new(text)", "Lexer::new(text, 0)")
    text = text.replace("Lexer::new(&src)", "Lexer::new(&src, 0)")
    with open(path, 'w', encoding='utf-8') as f:
        f.write(text)

replace_in_file('crates/apl_test/src/lib.rs')
replace_in_file('crates/apl_cli/src/main.rs')
replace_in_file('crates/apl_lsp/src/main.rs')
print('done')

import re

files_to_fix = [
    'crates/apl_parser/src/lib.rs',
    'crates/apl_sema/src/lib.rs'
]

for file_path in files_to_fix:
    with open(file_path, 'r', encoding='utf-8') as f:
        text = f.read()

    text = text.replace('Lexer::new(input)', 'Lexer::new(input, 0)')

    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(text)

print("done")

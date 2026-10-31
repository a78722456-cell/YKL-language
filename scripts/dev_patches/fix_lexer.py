import os

with open('crates/apl_lexer/src/lib.rs', 'r', encoding='utf-8') as f:
    lex = f.read()

lex = lex.replace(
'''pub struct Lexer<'a> {
    _input: &'a str,''',
'''pub struct Lexer<'a> {
    _input: &'a str,
    pub file_id: usize,'''
)
lex = lex.replace(
'''pub fn new(input: &'a str) -> Self {
        Self {
            _input: input,''',
'''pub fn new(input: &'a str, file_id: usize) -> Self {
        Self {
            _input: input,
            file_id,'''
)
with open('crates/apl_lexer/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(lex)
print('done')

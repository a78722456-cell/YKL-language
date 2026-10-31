import os

with open('crates/apl_ast/src/lib.rs', 'r', encoding='utf-8') as f:
    ast = f.read()

ast = ast.replace(
'''pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}''', 
'''pub struct Span {
    pub file_id: usize,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}'''
)

ast = ast.replace(
'''pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            start,
            end,
            line,
            col,
        }
    }''',
'''pub fn new(file_id: usize, start: usize, end: usize, line: usize, col: usize) -> Self {
        Self {
            file_id,
            start,
            end,
            line,
            col,
        }
    }'''
)
ast = ast.replace('Span::new(0, 0, 0, 0)', 'Span::new(0, 0, 0, 0, 0)')
with open('crates/apl_ast/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(ast)

with open('crates/apl_lexer/src/lib.rs', 'r', encoding='utf-8') as f:
    lex = f.read()

lex = lex.replace(
'''pub struct Lexer<'a> {
    input: &'a str,''',
'''pub struct Lexer<'a> {
    input: &'a str,
    file_id: usize,'''
)
lex = lex.replace(
'''pub fn new(input: &'a str) -> Self {
        Self {
            input,''',
'''pub fn new(input: &'a str, file_id: usize) -> Self {
        Self {
            input,
            file_id,'''
)
lex = lex.replace('Span::new(start_pos', 'Span::new(self.file_id, start_pos')
lex = lex.replace('Lexer::new(input)', 'Lexer::new(input, 0)')
with open('crates/apl_lexer/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(lex)
print('done')

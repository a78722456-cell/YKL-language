import os

with open('crates/apl_lexer/src/lib.rs', 'r', encoding='utf-8') as f:
    lex = f.read()

lex = lex.replace(
'''        let span = |end_pos, end_line, end_col| Span::new(self.file_id, start_pos, end_pos, end_line, end_col);''',
'''        let file_id = self.file_id;
        let span = |end_pos, end_line, end_col| Span::new(file_id, start_pos, end_pos, end_line, end_col);'''
)
with open('crates/apl_lexer/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(lex)
print('done')

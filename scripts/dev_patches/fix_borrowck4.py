import re

with open('crates/apl_borrowck/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# Add back * to lines 141, 152, 154
text = text.replace('errors.push(BorrowckError::MutationWhileBorrowed { span });', 'errors.push(BorrowckError::MutationWhileBorrowed { span: *span });')
text = text.replace('errors.push(BorrowckError::MutableBorrowConflict { span });', 'errors.push(BorrowckError::MutableBorrowConflict { span: *span });')
text = text.replace('errors.push(BorrowckError::ImmutableBorrowConflict { span });', 'errors.push(BorrowckError::ImmutableBorrowConflict { span: *span });')

with open('crates/apl_borrowck/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

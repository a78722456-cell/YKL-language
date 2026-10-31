import os
import re

with open('crates/apl_borrowck/src/points_to.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace('Statement, Statement', 'Statement')
text = text.replace('*kind', 'kind.clone()')

with open('crates/apl_borrowck/src/points_to.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

import re

with open('docs/testing/TESTING_STRATEGY.md', 'r', encoding='utf-8') as f:
    text = f.read()

# Mark some as checked
text = text.replace('[ ] orrowing/mutable_alias (Fail)', '[x] orrowing/mutable_alias (Fail)')
text = text.replace('[ ] orrowing/mutation_while_borrowed (Fail)', '[x] orrowing/mutation_while_borrowed (Fail)')
text = text.replace('[ ] orrowing/use_after_move (Fail)', '[x] orrowing/use_after_move (Fail)')
text = text.replace('[ ] orrowing/nll_dead_borrow_ok', '[x] orrowing/nll_dead_borrow_ok')

with open('docs/testing/TESTING_STRATEGY.md', 'w', encoding='utf-8') as f:
    f.write(text)

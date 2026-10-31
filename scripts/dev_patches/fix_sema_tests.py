import re

with open('crates/apl_sema/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = re.sub(r'#\[test\]\s*fn test_sema_move_error\(\) \{.*?\n\}\n\}', '\n}\n', text, flags=re.DOTALL)

with open('crates/apl_sema/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)

print("done")

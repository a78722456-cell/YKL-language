import re

with open('crates/apl_parser/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

def replace_span(match):
    start_obj = match.group(1).strip()
    args = match.group(2).strip()
    return f"Span::new({start_obj}.file_id, {start_obj}.start, {args})"

text = re.sub(r'Span::new\(\s*([a-zA-Z0-9_.]+(?:\.span|\.span_left)?)\.start\s*,\s*(.*?)\)', replace_span, text, flags=re.DOTALL)

with open('crates/apl_parser/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

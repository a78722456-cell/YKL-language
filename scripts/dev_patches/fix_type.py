import re
with open('crates/apl_ast/src/lib.rs', 'r', encoding='utf-8') as f:
    text = f.read()

impl_is_copy = '''    pub fn is_copy(&self) -> bool {
        match self {
            Type::Named(..) | Type::String | Type::Array(..) | Type::Slice(..) | Type::Channel(..) => false,
            _ => true,
        }
    }
'''

# insert it into impl Type { if it exists, else create impl Type {
if 'impl Type {' not in text:
    text = text.replace('impl std::fmt::Display for Type {', 'impl Type {\n' + impl_is_copy + '}\n\nimpl std::fmt::Display for Type {')
else:
    # not handling this branch for now since I know it doesn't exist
    pass

with open('crates/apl_ast/src/lib.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

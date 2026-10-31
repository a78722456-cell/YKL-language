import re

with open('crates/apl_mir/src/lowering.rs', 'r', encoding='utf-8') as f:
    text = f.read()

text = re.sub(
    r'pub struct MirBuilder \{[^}]*\}',
    '''pub struct MirBuilder {
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    current_block: BasicBlockId,
    loop_scopes: Vec<(BasicBlockId, BasicBlockId)>,
}''',
    text
)

text = text.replace('current_block: BasicBlockId(0),', 'current_block: BasicBlockId(0),\n            loop_scopes: Vec::new(),')

with open('crates/apl_mir/src/lowering.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print("done")

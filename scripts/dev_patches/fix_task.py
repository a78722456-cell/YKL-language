import os
path = r'C:\Users\Али\.gemini\antigravity\brain\87f1a180-33ae-4036-8c8f-570f7ef0a9f7\task.md'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

text = text.replace(
'''## YKL 0.4 - Semantic Compiler Core (CURRENT)
*Status: PLANNING/IMPLEMENTATION*''',
'''## YKL 0.4 - Semantic Compiler Core (CURRENT)
*Status: IMPLEMENTATION*'''
)

text = text.replace(
'''- [ ] **HIR Lowering**: Modify pl_sema to produce HIR (Name Resolution, Type Checking).
- [ ] **MIR Construction**: Transition MIR to strict CFG with BasicBlock and clear Predecessors/Successors graph.
- [ ] **Dataflow Framework**: Move borrow checking out of AST/Sema and into a MIR Dataflow Engine.''',
'''- [x] **HIR Definitions**: Defined fully resolved, ID-based pl_hir.
- [x] **HIR Lowering**: Created pl_hir::lower_program that performs Name Resolution without Ownership logic.
- [x] **HIR Invariants**: Added hir.validate() compiler checks.
- [x] **MIR Refactoring**: Re-architected pl_mir to be expression-less, using Place, Operand, Rvalue, and explicit CFG API.
- [x] **Tooling**: Added dump_mir and dump_cfg_dot (Graphviz) printers.
- [ ] **HIR Completion**: Finish full expression typing and module resolution.
- [ ] **MIR Lowering**: Translate HIR to the new CFG MIR.
- [ ] **Dataflow Framework**: Build Liveness and Borrow Checking on top of MIR Places.'''
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)
print('done')

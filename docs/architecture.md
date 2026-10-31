
# YKL Compiler Architecture (v0.4+)

This document describes the pipeline and invariants of the YKL Compiler.

## 1. The Pipeline
```
SOURCE -> Lexer -> Parser -> AST -> HIR -> MIR -> Dataflow Engine -> Backend (C/LLVM)
```

## 2. Invariants and IR Definitions

### 2.1 AST (Abstract Syntax Tree)
**Purpose:** Represents the syntactic structure of the source code.
**Invariants:**
- Every AST node must possess a valid `Span { file_id, start, end }`.
- Syntax is validated, but names and types are entirely unresolved.

### 2.2 HIR (High-Level Intermediate Representation)
**Purpose:** Strips away syntactic noise and resolves names and types. It is "boring" by design.
**Invariants:**
- All identifiers (functions, variables, types) are fully resolved (`pkg_name::func_name`).
- Every expression node is tagged with a fully resolved `Type`.
- There are no ambiguous generic parameters (after Monomorphization).

### 2.3 MIR (Mid-Level Intermediate Representation)
**Purpose:** Lowers control flow into a Control Flow Graph (CFG) consisting of BasicBlocks, Statements, and Terminators.
**Invariants:**
- `BasicBlock` contains a linear sequence of `Statement`s and ends with exactly one `Terminator` (e.g., `Return`, `Branch`, `CondBranch`).
- There are no nested expressions; complex expressions are flattened into sequential local variable assignments (`x = y + z`).
- `mir.validate()` must pass (no missing terminators, all locals declared).

## 3. The Dataflow Engine
Instead of writing isolated semantic passes, the YKL compiler utilizes a unified Dataflow Framework operating on the MIR CFG.

**Core Concept:**
```
Transfer(Block, State) -> State
```
The framework propagates `State` (e.g., Liveness, Initialization, Borrows) across the CFG edges until a fixed point is reached.

**Implementations:**
1. **LivenessAnalysis**: Computes which variables will be read later in the CFG.
2. **MoveAnalysis**: Computes whether a variable has been moved (Definite Initialization).
3. **BorrowChecker (NLL)**: Calculates Non-Lexical Lifetimes. A borrow is active only as long as its corresponding reference is Live according to `LivenessAnalysis`. If an invalidating action (like mutation) occurs while a borrow is active, an error is emitted via the `Span` preserved from HIR.


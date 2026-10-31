
# YKL Language Specification 0.1
## Chapter 1: Memory Model, Ownership, and Borrowing

### 1. Introduction
YKL (Young Kazah Language) is a safe, systems programming language designed with zero-cost abstractions, deterministic destruction, and explicit control over memory without a Garbage Collector. To achieve memory safety (absence of use-after-free, double-free, and dangling pointers) without runtime overhead, YKL employs a strict compile-time Ownership and Borrowing model.

### 2. Memory Model
The YKL memory model distinguishes between three primary storage durations:
1. **Stack**: All local variables, function arguments, struct values, and enum variants are allocated on the stack by default. YKL guarantees that structures are "flat" in memory.
2. **Heap**: Dynamic allocations must be explicit (e.g., via `alloc_array` or FFI `malloc`). Safe abstractions like `Vec<T>` manage heap memory internally and release it deterministically upon destruction (RAII).
3. **Static**: Global data, string literals, and constants.

### 3. Ownership Rules
Ownership in YKL is governed by three fundamental rules:
1. Each value in YKL has a variable that is called its **owner**.
2. There can only be **one owner at a time**.
3. When the owner goes out of scope, the value will be **dropped** (destroyed and memory freed).

#### 3.1. Move Semantics
Assignment (`let y = x;`) or passing a value to a function (`foo(x);`) transfers ownership if the type is not implicitly copyable.
- After a move, the original variable `x` becomes **uninitialized** (conceptually).
- Any attempt to use `x` after a move results in a compile-time error (`E_USE_AFTER_MOVE`).

#### 3.2. Copy Semantics
Types that trivially represent bytes and require no custom destruction logic (e.g., `i32`, `f64`, `bool`) are implicitly copied. Moves do not apply to them.

### 4. Borrowing Rules (References)
To allow temporary access to a value without transferring ownership, YKL provides references.
References are created using the `&` (immutable) and `&mut` (mutable) operators.

Borrowing is governed by the following rules (enforced by the Borrow Checker):
1. **At any given time, you can have EITHER one mutable reference (`&mut T`) OR any number of immutable references (`&T`).**
2. **References must always be valid.** (A reference cannot outlive its owner).

#### 4.1 Immutable Borrowing (`&T`)
An immutable borrow allows reading the data. While an immutable borrow exists:
- The owner cannot be moved.
- The owner cannot be mutated.
- The owner cannot be mutably borrowed.

#### 4.2 Mutable Borrowing (`&mut T`)
A mutable borrow allows modifying the data. While a mutable borrow exists:
- It holds exclusive access. No other references (mutable or immutable) can exist to the same data.
- The owner cannot be accessed directly (neither read nor written) until the mutable borrow expires.

### 5. Lifetimes (Internal Compiler Model)
While YKL users do not write explicit lifetime annotations (like `<'a>`), the compiler internally assigns a **Region** to every reference.
- **Scope**: The lexical block in which a variable is valid.
- **Loan**: A record of a borrow, tracking the borrowed path, the borrow kind (Shared or Mutable), and the region for which it is active.
- **Constraint Solver**: The Borrow Checker operates on MIR, calculating liveness for variables. A borrow is considered active as long as the reference it created is "live" (will be used later in the control flow graph).

If a loan conflicts with an action (e.g., mutation while a shared loan is active), the compiler issues `E_BORROW_CONFLICT`.

### 6. RAII and Deterministic Destruction
When a variable goes out of scope, its `Drop` logic (destructor) is deterministically invoked.
- For primitive types and flat structs, this is a no-op (stack pointer moves).
- For heap-allocated abstractions (`Vec<T>`), the destructor calls `free`.
- **Note:** Explicit `Drop` trait implementation is planned for future versions. Currently, standard library types hardcode cleanup logic.


# Young Kazah Language (YKL / APL) Specification 1.0

## 1. Introduction
Young Kazah Language (YKL, CLI `apl`) is a production-grade, systems-level general-purpose programming language designed to deliver C++ performance, Rust-level memory safety (without garbage collection), Go-like simplicity, and first-class concurrency.

## 2. Source Files & Lexical Grammar
- Source files use the `.yk` or `.apl` extension and must be UTF-8 encoded.
- Comments: Single-line `// comment`, multi-line `/* comment */`.
- Keywords: `fn`, `let`, `mut`, `if`, `else`, `match`, `return`, `struct`, `enum`, `trait`, `impl`, `async`, `await`, `spawn`, `channel`, `loop`, `while`, `for`, `in`, `pub`, `import`, `package`.

## 3. Type System
### 3.1 Primitive Types
- Integers: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `isize`, `usize`.
- Floating Point: `f32`, `f64`.
- Boolean: `bool` (`true`, `false`).
- Characters & Strings: `char` (Unicode code point), `string` (UTF-8 immutable string).

### 3.2 Algebraic Data Types (ADTs) & Pattern Matching
```yk
enum Option<T> {
    Some(T),
    None,
}

struct Point {
    x: f64,
    y: f64,
}
```

### 3.3 Nullable Analysis
Optional values can be represented ergonomically using `?T`.
```yk
let name: ?string = None;
```

## 4. Memory Safety & Ownership Model
YKL enforces compile-time memory safety without a Garbage Collector:
- **Ownership**: Each value has a single owner. When the owner goes out of scope, the value is dropped.
- **Borrowing**: References `&T` (immutable) and `&mut T` (mutable). Rules: either any number of immutable references OR exactly one mutable reference.
- **Allocators**: Built-in support for Arena, Bump, and Stack allocations via `@arena`, `@bump` annotations.

## 5. Concurrency Model
- **Async/Await**: Cooperative green threads managed by a work-stealing scheduler.
- **Actors & Channels**: Message passing via `channel<T>()`.
```yk
let (tx, rx) = channel<string>();
spawn async {
    tx.send("Hello from fiber!").await;
};
```

## 6. Standard Library Reference
- `std::collections`: `Vector<T>`, `HashMap<K, V>`, `HashSet<T>`, `RingBuffer<T>`.
- `std::io`: Console I/O, file reading/writing.
- `std::net`: TCP, UDP, HTTP server/client.
- `std::serialization`: JSON encoder/decoder.

## 7. Package Manager & CLI (`apl`)
- `apl init <name>`: Initializes a new project workspace.
- `apl build`: Compiles source code to machine executable or bytecode.
- `apl run`: Compiles and runs the entry point (`main`).
- `apl test`: Runs automated unit and property tests.


# YKL Language Specification 0.1
## Chapter 2: Type System, Generics, and Data Types

### 1. Introduction
YKL features a strong, static type system with algebraic data types (ADTs) and parametric polymorphism (Generics). The type system is designed to provide strict guarantees at compile time while mapping directly to C/machine types with zero overhead.

### 2. Primitive Types
- **Integers**: `i8`, `i16`, `i32`, `i64`, `isize` (signed), `u8`, `u16`, `u32`, `u64`, `usize` (unsigned).
- **Floats**: `f32`, `f64`.
- **Boolean**: `bool` (`true`, `false`).
- **Characters/Strings**: `char`, `string` (internal string literals).

### 3. Composite Types
#### 3.1. Structs
Structs define composite records. They are statically laid out in memory as flat C structs.
```rust
struct Point {
    x: i32,
    y: i32,
}
```

#### 3.2. Enums (Algebraic Data Types)
Enums in YKL are tagged unions. A variant may hold data.
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```
Internally, an enum is represented as a C struct containing an integer tag and a union of all possible variant data layouts. The overall size is the size of the tag plus the size of the largest variant (plus alignment padding).

### 4. Generics (Parametric Polymorphism)
Structs, Enums, and Functions can take type parameters.
```rust
struct Vec<T> {
    data: Slice<T>,
    len: usize,
    cap: usize,
}
```
YKL resolves generics through **Monomorphization**. For every unique combination of concrete types used, the compiler generates a separate, specialized copy of the struct/function. 
- **Advantage**: Maximum runtime performance and complete absence of type erasure or vtables.
- **Trade-off**: Increases compilation time and binary size (code bloat).

### 5. Type Inference
The YKL compiler performs local type inference.
- Variable bindings (`let x = 5;`) do not require explicit type annotations if the expression provides a concrete type.
- Function signatures **must** have explicit types for parameters and return values.

### 6. Subtyping and Coercions
YKL explicitly avoids most implicit conversions (e.g., `i32` to `i64` requires explicit casting).
- **Reference Coercions**: `&mut T` implicitly coerces to `&T` when passed to a function expecting `&T`.
- **Pointer Coercions**: Arrays and Slices implicitly coerce to raw `Ptr` when interacting with FFI `extern "C"` functions (specifically designed for memory intrinsics).


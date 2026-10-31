
# YKL Language Specification 0.1
## Chapter 3: Execution, ABI, and FFI

### 1. The Execution Model
YKL execution strictly begins at the `main` function.

### 2. ABI and Memory Layout
YKL makes a strict distinction between **Semantic Representation** and **Physical Representation**. 

#### 2.1 Default YKL Struct Layout
By default, the YKL compiler is completely free to reorder fields in a structure to minimize padding and optimize memory layout. The default layout is *not* guaranteed to be stable across compiler versions or compatible with C.

#### 2.2 Default YKL Enum Layout
By default, Enums are implemented as Tagged Unions. However, the compiler reserves the right to perform **Niche Optimization**. 
For example, in `enum Option<&T> { Some(&T), None }`, the compiler knows that a valid reference `&T` cannot be NULL. Therefore, it can encode `None` as the `0` (NULL) pointer value, eliminating the need for a separate tag field entirely. This allows `Option<&T>` to have the exact same physical size as `&T`.

#### 2.3 Strict C Compatibility (`@repr(C)`)
When interoperability with C or stable binary layout is required, developers must explicitly use the `@repr(C)` attribute.
```rust
@repr(C)
struct CUser {
    id: u64,
    age: u32,
}
```
For `@repr(C)` types, the YKL compiler mathematically guarantees that the memory layout, padding, and alignment match the standard C ABI (e.g., System V ABI for x86_64) of the target platform. Field reordering and niche optimizations are strictly disabled for these types.

### 3. FFI (Foreign Function Interface)
YKL supports calling external C functions seamlessly via `extern` blocks.

```rust
extern {
    fn malloc(size: usize) -> Ptr;
    fn free(ptr: Ptr) -> i32;
}
```
- Only `@repr(C)` structs and primitive types (like `i32`, `Ptr`) are safe to pass across FFI boundaries.


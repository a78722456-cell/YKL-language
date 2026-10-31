# YKL Testing Strategy and Coverage Matrix

This document outlines the comprehensive testing strategy for the YKL compiler, moving from prototype to a production-ready systemic language.

## 0. Organization of Test Tree

The tests are organized into the following directory structure:

`	ext
tests/
+-- lexer/
+-- parser/
+-- ast/
+-- resolution/
+-- types/
+-- generics/
+-- traits/
+-- adt/
+-- patterns/
+-- expressions/
+-- statements/
+-- ownership/
+-- borrowing/
+-- lifetimes/
+-- initialization/
+-- control_flow/
+-- hir/
+-- mir/
+-- cfg/
+-- dataflow/
+-- drops/
+-- codegen/
+-- vm/
+-- differential/
+-- ffi/
+-- abi/
+-- cli/
+-- diagnostics/
+-- std/
+-- integration/
+-- compile-pass/
+-- compile-fail/
+-- fuzz/
+-- property/
+-- regression/
+-- performance/
L-- security/
`

## 1. Lexer - FULL SUITE
- [ ] lexer/identifier_basic
- [ ] lexer/identifier_underscore
- [ ] lexer/identifier_numbers
- [ ] lexer/identifier_unicode
- [ ] lexer/identifier_long
- [ ] lexer/keyword_as_identifier (Fail)
- [ ] lexer/integer_decimal
- [ ] lexer/integer_hex
- [ ] lexer/integer_binary
- [ ] lexer/integer_with_underscores
- [ ] lexer/float_exponent
- [ ] lexer/string_unicode
- [ ] lexer/string_escaped
- [ ] lexer/string_unterminated (Fail)

## 2. Parser
- [ ] parser/struct_nested
- [ ] parser/enum_with_data
- [ ] parser/impl_blocks
- [ ] parser/missing_semicolon (Fail)
- [ ] parser/unmatched_braces (Fail)

## 3. HIR & Name Resolution
- [ ] 
esolution/shadowing_basic
- [ ] 
esolution/shadowing_nested_scopes
- [ ] 
esolution/use_before_decl (Fail)
- [ ] 
esolution/undefined_symbol (Fail)

## 4. Types
- [ ] 	ypes/inference_basic
- [ ] 	ypes/mismatch_assignment (Fail)
- [ ] 	ypes/mismatch_return (Fail)

## 5. Control Flow & MIR Lowering
- [ ] mir/loop_basic
- [ ] mir/while_nested
- [ ] mir/break_continue
- [ ] mir/break_outside_loop (Fail)
- [ ] mir/short_circuit_and
- [ ] mir/short_circuit_or

## 6. Dataflow & Initialization
- [ ] initialization/use_uninit (Fail)
- [ ] initialization/conditionally_init (Fail)
- [ ] initialization/init_in_loop

## 7. Borrowing & Ownership
- [x] orrowing/mutable_alias (Fail)
- [ ] orrowing/immutable_alias_ok
- [x] orrowing/mutation_while_borrowed (Fail)
- [x] orrowing/use_after_move (Fail)
- [x] orrowing/nll_dead_borrow_ok

## 8. Compiler Gates & Readiness

### Gate A: Frontend Correctness
All syntax is correctly parsed and rejected if invalid. Diagnostics point exactly to the error span.

### Gate B: Semantic & Ownership
Name resolution, type inference, and borrow checking (NLL) are mathematically sound via dataflow on the CFG.

### Gate C: MIR Transformations
MIR is correctly flattened into BasicBlocks. Drops are injected at scope exits accurately.

### Gate D: Codegen & VM
The C Backend and Virtual Machine correctly execute the lowered MIR. Differential fuzzing between VM and C backend yields identical results.

# WASM RT Milestones

## MS1

> goal for first half of the lecture period (~end of May)

 - Spec compliant parser and validator
 - Interpreter backend
 - Spec test e2e testing

## MS2

> goal for end of lecture period (~mid of July)

 - Translator to LLVM + LLVM code generation backend
 - Support for the following extensions:
    - Memory64
    - Fixed-width SIMD
 - WASI Support
 - Limited set of benchmarks

## MS3

> goal for the end of the semester (~begin of October)

 - x86_64 direct emit code generation backend
 - JIT compilation
 - Support for the following extensions:
    - Multible Memories
    - Threads and Atomics
    - Relaxed SIMD
    - Sign-extension Operations

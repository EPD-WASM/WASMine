TODO:
 - WASI: Only stubs available, method signature needs to be changed.
    - Subset: ~until 27.06. by Jakob~ => bis 11.07.: args_get, args_sizes_get, environ_get, environ_sizes_get, fd_close, fd_fdstat_get, fd_prestat_get, fd_prestat_dir_name, fd_fdstat_set_flags, path_open, path_filestat_get, fd_read, fd_seek, fd_write, proc_exit, sched_yield
    - with simple unit tests
    - full support TBD
 - Runtime: Tables
    - basic support for indirect calls exists, but missing instructions
    - implement missing instructions ~by Jakob until 27.06.~ => 11.07.
 - Imports:
    - completely missing -> Lukas until 27.06.
 - Interpreter
    - missing: table instructions, imports
    - ~until 27.06. 90% of the spec tests by Enrico~ =>
 - LLVM Backend
    - make tests pass: 90% until 27.06. by Lukas
 - Vector Support (+ SIMD)
 - Traps in Runtime:
    - Lukas until 11.07. => use siglongjump
    - completely missing, need to be implemented until 27.06. by however needs them first.
 - Benchmarks:
    - Sightglass -> write adapter for our runtime
    - SpecCPU -> compile
    - Add to criterion benchmarks more runtimes: wasmtime, wasmer, (aWsm), wasm3, wasmedge



Alexis fragen:
 - Internal Call conv? Wrapper for exports(internal) -> exports(C) + imports(C) -> imports(internal)?
 - Where could ORC Symbols be lost?
 - can be leak function addresses to wasm code? e.g. ref.func -> store to global -> import global in other module -> call to ref
 - internal call convention fast cc vs c cc

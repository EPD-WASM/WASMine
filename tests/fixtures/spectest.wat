;; WebAssembly spec test default module: https://github.com/WebAssembly/spec/blob/d73ac324e6ace2a190bf9a1af4ebd1638dd26d48/interpreter/README.md?plain=1#L433
(module
  (global (export "global_i32") i32 (i32.const 666))
  (global (export "global_i64") i64 (i64.const 666))
  (global (export "global_f32") f32 (f32.const 666.6))
  (global (export "global_f64") f64 (f64.const 666.6))

  (table (export "table") 10 20 funcref)

  (memory (export "memory") 1 2)

  (func (export "print")
    (i32.const 0)
    (drop)
  )
  (func (export "print_i32") (param $i i32)
    (i32.const 0)
    (drop)
  )
  (func (export "print_i64") (param $i i64)
    (i32.const 0)
    (drop)
  )
  (func (export "print_f32") (param $i f32)
    (i32.const 0)
    (drop)
  )
  (func (export "print_f64") (param $i f64)
    (i32.const 0)
    (drop)
  )
  (func (export "print_i32_f32") (param $i i32) (param $j f32)
    (i32.const 0)
    (drop)
  )
  (func (export "print_f64_f64") (param $i f64) (param $j f64)
    (i32.const 0)
    (drop)
  )
)
(module
  (type (;0;) (func (param i32)))
  (func (;0;) (type 0) (param i32)
    local.get 0
    i32.eqz
    f64.convert_i32_u
    i32.const -129
    i32.const -134217728
    if (param i32)  ;; label = @1
      block (param i32)  ;; label = @2
        global.get 0
        i32.xor
        global.set 0
      end
    else
      unreachable
    end
    i64.reinterpret_f64
    global.get 1
    i64.xor
    global.set 1)
  (global (;0;) (mut i32) (i32.const 0))
  (global (;1;) (mut i64) (i64.const 0))
  (export "" (global 0)))

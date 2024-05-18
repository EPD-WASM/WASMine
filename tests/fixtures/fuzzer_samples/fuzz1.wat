(module
  (type (;0;) (func (param i64 i32)))
  (func (;0;) (type 0) (param i64 i32)
    local.get 0
    local.tee 0
    i64.extend16_s
    i32.const 36438624
    block (param i64 i32)  ;; label = @1
      i32.extend8_s
      global.get 0
      i32.xor
      global.set 0
      global.get 1
      i64.xor
      global.set 1
    end)
  (func (;1;) (type 0) (param i64 i32)
    (local i32))
  (func (;2;) (type 0) (param i64 i32))
  (func (;3;) (type 0) (param i64 i32))
  (func (;4;) (type 0) (param i64 i32))
  (global (;0;) (mut i32) (i32.const 0))
  (global (;1;) (mut i64) (i64.const 0)))

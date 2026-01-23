(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add)
  (func $sub (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.sub)
  (export "add" (func $add))
  (export "sub" (func $sub))
)
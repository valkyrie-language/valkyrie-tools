;; Simple WebAssembly Component for testing
;; This is a WAT representation that should be more compatible

(component
  ;; Define interface types
  (type (func (param "a" u32) (param "b" u32) (result u32)))
  
  ;; Core module
  (core module $math
    (func $add (param i32) (param i32) (result i32)
      local.get 0
      local.get 1
      i32.add)
    
    (func $sub (param i32) (param i32) (result i32)
      local.get 0
      local.get 1
      i32.sub)
    
    (export "add" (func $add))
    (export "sub" (func $sub))
  )
  
  ;; Instantiate core module
  (core instance $math_inst (instantiate (core module $math)))
  
  ;; Export functions
  (export "add" (func (type 0) (canon.lift (core func $math_inst "add"))))
  (export "sub" (func (type 0) (canon.lift (core func $math_inst "sub"))))
)
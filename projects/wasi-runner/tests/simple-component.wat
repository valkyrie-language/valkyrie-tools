;; WebAssembly Component example
;; This is a component with a WIT interface, not just a module

(component
  ;; Define a simple interface
  (type $add_interface (func (param "a" u32) (param "b" u32) (result u32)))
  
  ;; Create a module with the implementation
  (module $math_module
    (func $add (param $a i32) (param $b i32) (result i32)
      local.get $a
      local.get $b
      i32.add)
    
    (func $sub (param $a i32) (param $b i32) (result i32)
      local.get $a
      local.get $b
      i32.sub)
    
    (export "add" (func $add))
    (export "sub" (func $sub))
  )
  
  ;; Instantiate the module
  (instance $math_instance (instantiate (module $math_module)))
  
  ;; Export the functions with component interface
  (export "add" (func $math_instance "add" (type $add_interface)))
  (export "sub" (func $math_instance "sub" (type $add_interface)))
)
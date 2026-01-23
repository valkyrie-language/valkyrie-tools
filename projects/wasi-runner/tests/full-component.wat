;; 完整的WebAssembly Component示例
;; 使用组件模型规范创建

(component
  ;; 定义组件类型
  (type $add_func (func (param u32 u32) (result u32)))
  (type $sub_func (func (param u32 u32) (result u32)))
  
  ;; 定义组件接口
  (component $math_component
    (import "add" (func (param u32 u32) (result u32)))
    (import "sub" (func (param u32 u32) (result u32)))
    
    (export "add" (func 0))
    (export "sub" (func 1))
  )
  
  ;; 创建实现模块
  (module $math_impl
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
  
  ;; 实例化实现模块
  (instance $math_inst (instantiate (module $math_impl)))
  
  ;; 创建组件实例
  (instance $comp_inst (instantiate (component $math_component)
    (with "add" (func $math_inst "add"))
    (with "sub" (func $math_inst "sub"))
  ))
  
  ;; 导出组件接口
  (export "add" (func $comp_inst "add"))
  (export "sub" (func $comp_inst "sub"))
)
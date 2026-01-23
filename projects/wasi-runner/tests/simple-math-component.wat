;; 简化的WebAssembly Component
;; 符合组件模型规范

(component
  ;; 定义函数类型
  (type $math_func (func (param u32 u32) (result u32)))
  
  ;; 创建核心模块
  (core module $math
    (func $add (export "add") (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.add)
    
    (func $sub (export "sub") (param i32 i32) (result i32)
      local.get 0
      local.get 1
      i32.sub)
  )
  
  ;; 实例化核心模块
  (core instance $math_inst (instantiate (core module $math)))
  
  ;; 定义组件函数（lift核心函数到组件层）
  (func $add_comp (type $math_func) (canon lift (core func $math_inst "add")))
  (func $sub_comp (type $math_func) (canon lift (core func $math_inst "sub")))
  
  ;; 导出组件函数
  (export "add" (func $add_comp))
  (export "sub" (func $sub_comp)))
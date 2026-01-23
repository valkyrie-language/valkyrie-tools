;; 正确的WebAssembly Component示例
;; 使用标准的组件模型语法

(component
  ;; 定义类型
  (type $add_ty (func (param "a" u32) (param "b" u32) (result u32)))
  (type $sub_ty (func (param "a" u32) (param "b" u32) (result u32)))
  
  ;; 创建核心模块
  (core module $math_module
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
  
  ;; 实例化核心模块
  (core instance $math_inst (instantiate (core module $math_module)))
  
  ;; 创建组件函数（lift到组件层）
  (func $add_comp (type $add_ty) (canon lift (core func $math_inst "add")))
  (func $sub_comp (type $sub_ty) (canon lift (core func $math_inst "sub")))
  
  ;; 导出组件函数
  (export "add" (func $add_comp))
  (export "sub" (func $sub_comp))
)
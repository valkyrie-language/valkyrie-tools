### 阶段 1：基于 Rust 的原型（6个月）
```
目标：实现可工作的原型，验证核心概念
季度1（第1-3月）：
1. [x] 实现词法分析器（lexer）
2. [x] 实现语法分析器（parser）
3. [x] 构建基本 AST 结构
4. [x] 实现符号表（Environment）与 namespace/using 声明解析

季度2（第4-6月）：
1. [x] 实现类型检查器（HIR 基础类型推断/检查：已可运行）
3. [x] 实现 trait/class 解析 (AST/HIR/MIR 全链路打通，支持 OOP 多态与继承、静态成员、构造函数重载)
4. [x] 构建 HIR 层 (基本结构、Lowering、符号解析)
5. [x] 实现基本的宏展开系统 (AST 解析完成, 运行时支持自定义宏调用)
6. [x] 解释器原型（执行 MIR） - (AST 解释器已完成，基于 Stack 的 MIR VM 已初步实现)
7. [x] 实现异步编程支持 (async/await) 与 生成器 (yield)

里程碑：能运行 "Hello World" 和基本算术运算
```

### 阶段 2：完整编译管线（9个月）
```
目标：实现完整的编译和执行管线
季度1（第7-9月）：
1. [x] 完善宏系统（编译期函数、AST 变换）
2. [x] 实现注记系统
3. [x] 构建 HIR 层 (基本结构与 Lowering)
4. [x] 实现基础优化通道

季度2（第10-12月）：
1. [x] 实现部分求值器 (Partial Evaluator)
2. [x] 构建 JIT 执行引擎 (MIR Compiler & VM 已实现)
3. [x] 实现基础标准库 (Native Function 支持，std::io, std::math)
34. [x] 实现数组/列表支持 (List Literals, Indexing 1-based/0-based)
35. [x] 添加调试支持 (Source Mapping & Stack Trace Infrastructure)
36. [x] 重构 VM 为迭代式栈机 (Iterative Stack-based VM to prevent stack overflow) - Fixed stack management for function returns and constructors

季度3（第13-15月）：
37. [x] 支持 Rust 风格的隐式 Return (Implicit return for blocks/functions/if-expressions)
38. [x] 区分 REPL 模式与 Project 模式 (Implemented `vcc repl`, `vcc check`, `vcc run` with `ValkyrieSession` state persistence and top-level let promotion)
39.41. [x] 实现 WASM 后端 + WIT COMPONENT FFI (Basic arithmetic & Locals supported; Control Flow pending)
42. [x] 实现包管理器原型 (Implemented in `library/pkg` with TOML support) [x] 构建测试框架 (Implemented in `library/test` and `vcc test` command, supported `@.test`, `@assert`, `@debug`)
42. [ ] 性能优化

里程碑：能编译和执行 Valkyrie 标准库
```

### 阶段 3：自举准备（6个月）
```
目标：用 Valkyrie 重写编译器核心组件
50. 季度1（第16-18月）：
51. [x] 语言特性完善：Control Flow (While Loop, Block Expressions), Collections (List/String methods)
52. [x] 用 Valkyrie 重写 lexer (Prototype completed in `library/ast/lexer.vk`)
53. [x] 用 Valkyrie 重写 parser (Prototype completed in `library/ast/parser/_.vk` - Fully adopted Pratt Parser architecture for enhanced stability and operator support)
54. [x] 实现编译器自举工具链 (Prototype `binary/vcc.vk` running)
55. [x] 重构编译器库结构 (Namespace reorganization completed: `library/ast`, `library/hir`, etc.)
56. [x] 移除 `@include` 依赖，全面采用 `using package::` 模块系统
57. [x] 建立交叉编译能力 (Implemented WASM cross-compilation target in bootstrapping compiler)
58. [x] 完善错误处理和诊断 (Implemented `Diagnostic` class and integrated into TypeChecker/SemanticAnalyzer)

季度2（第19-21月）：
58. [x] 用 Valkyrie 重写语义分析器 (Prototype in `library/hir/semantic/_.vk`)
59. [x] 用 Valkyrie 重写 HIR Lowering (Prototype in `library/hir/lowering.vk`)
60. [x] 用 Valkyrie 重写类型检查器 (Prototype in `library/hir/semantic/typecheck.vk` - Supports primitive types i32/f32/bool/() and Generics)
61. [x] 用 Valkyrie 重写 MIR 生成 (Prototype in `library/mir/compiler.vk`)
62. [x] 实现 MIR 解释器 (Bytecode Interpreter in `library/mir/interpreter.vk`)
63. [x] 基础类型重构 (i32, f32, (), String, etc.)
66. [x] 泛型支持 (AST/Parser/HIR Support)
67. [x] Result 类型 (std::result::Result)
68. [x] 用 Valkyrie 重写宏系统
69. [x] 实现命令行交互能力 (CLI Args, Env Vars, CWD)
70. [x] 优化模块解析 (Nested Using, Namespace Package Prefix, Wildcard Imports)
71. [x] 修复 Rust 宿主编译错误与完善错误处理 trait
72. [x] 数据结构增强 (Map 支持, 基于 IndexMap<String, Object>)
73. [x] 实现完整的自举流程 (Self-compilation verified: `vcc` compiles itself to WASM)
74. [x] 验证自举正确性 (Bootstrapping loop functional)
75. [x] 实现二进制与库的链接 (Binary linking against Library via `using package::`)
76. [x] 增强标准库集合 (Strong-typed List<T>, HashMap<K,V>)
77. [x] 实现有序映射 TreeMap<K,V> (Prefix Ordered) - (Skeleton implemented in `valkyrie-std`)
78. [x] 实现插入序映射 IndexMap<K,V> (Insertion Ordered) - (Skeleton implemented in `valkyrie-std`)
79. [x] 跨项目解析支持 (Support for `valkyrie-std` via relative imports)
80. [x] 实现 List<T> 和 Array<T> 在 valkyrie-std 中的定义 (Refactored to `LinkedList<T>`, `Array<T>`, `ArrayList<T>` and traits)
81. [x] 迁移 Result 类型至 valkyrie-std 并重构为 Fine/Fail
82. [x] 注册基础类型 (i32, i64, bool, char, etc.) 到 valkyrie-std
83. [x] 重构 std::os 模块并支持多后端 FFI (@.native 注记)
84. [x] 实现 WASM 二进制直接生成 (Direct WASM binary generation via `CodegenWasm` in `vcc`)


里程碑：能用 Valkyrie 编译简单的 Valkyrie 程序 (Classes, Functions, Control Flow, Type Annotations, Generics supported)
```

### 阶段 4：完全自举与多后端（12个月）
```
目标：实现完全自举并支持多目标平台
季度1（第22-24月）：
83. [x] 完全自举编译器 (Self-hosting verified: vcc compiles itself to WASM)
84. [x] 优化编译性能 (Reduced I/O overhead in Semantic Analysis and Type Checking)
85. [x] 用 valkyrie 实现 WASM 后端 + WIT COMPONET FFI (Initial implementation in `codegen_wasm.vk`)

季度2（第25-27月）：
88. [x] 用 valkyrie 实现 JVM 后端 (Prototype completed in `codegen_jvm.vk`, generates Jasmin assembly)
89. [x] 用 valkyrie 实现 C 后端 (Prototype completed in `codegen_c.vk`, generates C99 code)
90. [ ] 用 valkyrie 实现 CLR 后端
91. [ ] 用 valkyrie 实现 Native 后端（LLVM）
92. [ ] 用 valkyrie 构建 LSP 插件原型

季度3（第28-30月）：
94. [ ] 用 valkyrie 实现并发和并行支持
95. [ ] 用 valkyrie 构建 FFI 系统
96. [ ] 用 valkyrie 实现性能分析工具

季度4（第31-33月）：
99. [ ] 用 valkyrie 完善生态系统工具
100. [ ] 用 valkyrie 构建文档系统
101. [ ] 优化多后端代码生成
```

# Valkyrie Language Bootstrap Compiler

这是一个用 JavaScript 实现的 Valkyrie 语言自举编译器，可以将 `.valkyrie` 源代码编译为 JavaScript。

## 🚀 快速开始

### 安装依赖

```bash
npm install
```

### 编译示例

```bash
# 运行自举编译
node bootstrap.js --bootstrap
```

### 运行测试

```bash
npm test
```

### 开发模式（监听文件变化）

```bash
npm run watch
```

## 📁 项目结构

```tree
valkyrie-bootstrap/
├── bootstrap.js          # 自举编译脚本
├── dist/                 # 编译产物（自举生成）
│   ├── cli.js
│   ├── codegen.js
│   ├── lexer.js
│   └── parser.js
├── library/              # Valkyrie 源代码
│   ├── cli.valkyrie
│   ├── codegen.valkyrie
│   ├── lexer.valkyrie
│   └── parser.valkyrie
├── package.json          # 项目配置
├── readme.md             # 项目说明
└── test/                 # 测试文件
    ├── debug-all-files.js
    ├── debug-lexer-only.js
    ├── debug-semantic-detail.js
    └── test.js
```

## 🔧 编译器架构

### 编译流程

1. **词法分析 (Lexer)** - 将源代码转换为 token 流
2. **语法分析 (Parser)** - 将 token 流转换为抽象语法树 (AST)
3. **语义分析 (Semantic)** - 进行类型检查和符号表管理
4. **代码生成 (CodeGen)** - 将 AST 转换为 JavaScript 代码

### 核心组件

- **Lexer**: 词法分析器，识别关键字、标识符、字面量等
- **Parser**: 语法分析器，构建 AST
- **CodeGenerator**: 代码生成器，输出 JavaScript 代码
- **ValkyrieCompiler**: 简化的编译器，整合词法分析、语法分析和代码生成

## 🎯 Valkyrie 语言特性

### 基础特性

- ✅ 类和继承
- ✅ 函数和方法
- ✅ 变量和常量
- ✅ 基本数据类型 (string, number, boolean)
- ✅ 数组和对象字面量
- ✅ 控制流 (if/else, for, while)

### 高级特性

- ✅ 命名空间 (namespace)
- ✅ 枚举 (enum)
- ✅ 联合类型 (union)
- ✅ 接口/特征 (trait)
- ✅ 静态方法和属性
- ✅ 构造函数
- ✅ Lambda 表达式

### 类型系统

- ✅ 类型注解
- ✅ 泛型支持（基础）
- ✅ 类型推断（部分）

## 📝 语法示例

### 基础类定义

```valkyrie
namespace MyApp

class Person {
    name: string
    age: number
    
    constructor(name: string, age: number) {
        this.name = name
        this.age = age
    }
    
    greet() {
        console.log("Hello, " + this.name)
    }
}
```

### 枚举和联合类型

```valkyrie
enum Color {
    Red,
    Green,
    Blue
}

union Result {
    Success(value: string),
    Error(message: string, code: number)
}
```

### 接口/特征

```valkyrie
trait Drawable {
    draw(): void
    getArea(): number
}

class Circle implements Drawable {
    radius: number
    
    draw() {
        console.log("Drawing circle")
    }
    
    getArea(): number {
        return 3.14159 * this.radius * this.radius
    }
}
```

## 🛠️ 命令行使用

### 运行自举编译

```bash
node bootstrap.js --bootstrap
```

## 🧪 测试

项目包含完整的测试套件，覆盖编译器的各个组件：

```bash
# 运行所有测试
npm test

# 测试包括：
# - 词法分析器测试
# - 语法分析器测试
# - 语义分析器测试
# - 代码生成器测试
# - 端到端编译测试
# - 错误处理测试
```

## 🔍 示例文件

### basic.valkyrie
展示 Valkyrie 语言的基础特性，包括类、继承、函数等。

### advanced.valkyrie
展示高级特性，包括枚举、联合类型、trait、泛型等。

### namespace.valkyrie
展示命名空间机制和多文件项目结构。

## 🚧 开发状态

当前版本是 Valkyrie 语言的自举编译器原型，已实现核心语言特性，并成功完成自举。

### 已实现
- ✅ 完整的自举编译流程
- ✅ 基础语法支持
- ✅ 命名空间机制
- ✅ 类型系统（基础）
- ✅ 代码生成到 JavaScript
- ✅ 测试套件

### 计划中
- 🔄 更完善的类型检查
- 🔄 模块系统
- 🔄 更好的错误报告
- 🔄 优化和性能改进
- 🔄 IDE 支持
- 🔄 命令行工具 (CLI) 的重新实现

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！

## 📄 许可证

ISC License

---

**Valkyrie Language Bootstrap Compiler** - 用 JavaScript 实现的 Valkyrie 语言编译器
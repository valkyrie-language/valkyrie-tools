# Legion - Valkyrie Workspace Manager

Legion 是一个纯 Valkyrie 实现的 workspace 管理器，用于管理 Valkyrie 项目的构建、依赖和开发工作流。

## 特性

- 🚀 **纯 Valkyrie 实现** - 使用 Valkyrie 语言编写，展示语言特性
- 📦 **依赖管理** - 自动安装和管理项目依赖
- 🔨 **智能构建** - 增量编译和依赖跟踪
- 👀 **文件监听** - 自动重新构建变化的文件
- 📁 **项目模板** - 快速创建新项目
- ⚡ **高性能** - 并行编译和优化构建流程

## 安装

```bash
# 在自举阶段使用 npm 安装依赖
npm install

# 使用 bootstrap 脚本运行
npm run bootstrap -- build
```

## 使用方法

### 基本命令

```bash
# 构建项目
legion build

# 监听文件变化
legion watch

# 清理构建产物
legion clean

# 初始化新项目
legion init my-project

# 显示帮助
legion help

# 显示版本
legion --version
```

### 高级选项

```bash
# 指定配置文件
legion build --config custom.yml

# 指定输出目录
legion build --output out

# 启用详细输出
legion build --verbose

# 构建并监听变化
legion build --watch
```

### 项目结构

Legion 支持灵活的项目结构，自动识别 library 和 binary 文件：

```
my-project/
├── legion.yml          # 项目配置文件
├── library/            # 库文件目录（只有一个）
│   └── example.valkyrie
├── binary/             # 二进制文件目录（可以有多个）
│   └── legion/
│       └── main.vk
├── show.vk            # 单独的二进制文件
├── dist/               # 构建输出目录
│   ├── index.js       # 导出库函数
│   ├── legion.js      # 可以导入 index.js
│   └── show.js        # 可以导入 index.js
└── node_modules/      # 依赖目录
```

### 配置文件 (legion.json)

Legion 使用 JSON 格式的配置文件，遵循**约定大于配置**的原则，大部分设置都有合理的默认值：

```json
{
  "name": "my-project",
  "version": "1.0.0",
  "source": "src",
  "build": "build", 
  "main": "src/main.vk",
  "target": "javascript",
  "dependencies": [
    "@valkyrie-language/stdlib@latest"
  ]
}
```

**约定大于配置**: Legion 会自动识别项目结构，大部分情况下无需复杂配置。只有在需要自定义构建行为时才需要修改配置文件。

## 开发状态

Legion 目前处于开发阶段，使用 bootstrap.js 提供基本的自举功能，直到 Valkyrie 编译器完全可用。

### 自举阶段

在自举阶段，我们使用 JavaScript 辅助脚本来提供基本功能：

- `bootstrap.js` - 提供基本的构建和项目管理功能
- 引用线上的 `valkyrie-language/valkyrie-bootstrap` 进行编译
- 一旦 Valkyrie 编译器成熟，将完全迁移到纯 Valkyrie 实现

### 模块结构

```
src/
├── main.vk           # 主入口点
├── config.vk         # 配置管理
├── compiler.vk       # 编译器接口
├── dependency.vk     # 依赖管理
└── workspace.vk      # 工作空间管理
```

## Workspace 管理器

Legion 是一个智能的 workspace 管理器，能够自动识别项目结构并生成对应的 JavaScript 输出。

### 包结构说明

Legion 采用标准的包管理器结构，支持以下目录类型：

#### 📚 Library 目录
- **位置**: `library/` 目录
- **作用**: 存放库代码，会被编译成可重用的模块
- **输出**: `dist/index.js` - 导出所有库函数
- **使用**: 可以被 binary 和 test 代码导入使用

#### ⚙️ Binary 目录（可选）
- **位置**: `binary/` 目录
- **作用**: 存放可执行程序代码
- **结构支持**:
  - `binary/cmd_name.vk` - 单文件程序，需要包含 `main` 函数
  - `binary/cmd_name/main.vk` - 目录程序，`main.vk` 文件中需要包含 `main` 函数
- **输出**: `dist/cmd_name.js` - 对应的可执行文件
- **要求**: 必须定义 `main` 函数作为程序入口

#### 🧪 Test 目录（可选）
- **位置**: `test/` 目录
- **作用**: 存放测试代码
- **输出**: `dist/test/index.js` - 测试套件
- **特点**: 通常用于单元测试和集成测试

#### 📦 Dist 目录（自动生成）
- **位置**: `dist/` 目录
- **作用**: 构建输出目录
- **内容**:
  - `dist/index.js` - library 编译结果
  - `dist/cmd_name.js` - binary 程序编译结果
  - `dist/test/index.js` - test 编译结果
- **特点**: 该目录由构建系统自动生成，不应手动修改

### 编译规则

#### Library 处理
- **位置**: `library/` 目录
- **文件扩展名**: `.valkyrie`
- **输出**: 编译为 `dist/index.js`，导出所有库函数
- **索引**: 自动生成导出模块，方便其他代码导入

#### Binary 处理

**单文件类型**:
- **位置**: `binary/cmd_name.vk`
- **要求**: 文件中必须定义 `main` 函数作为入口点
- **输出**: 编译为 `dist/cmd_name.js`
- **依赖**: 自动导入 `dist/index.js`（如果存在 library）

**目录类型**:
- **位置**: `binary/cmd_name/main.vk`
- **要求**: `main.vk` 文件中必须定义 `main` 函数
- **输出**: 编译为 `dist/cmd_name.js`
- **依赖**: 自动导入 `dist/index.js`（如果存在 library）

#### Test 处理
- **位置**: `test/` 目录
- **输出**: 编译为 `dist/test/index.js`
- **用途**: 用于运行测试套件

### 输出结构示例

```
dist/
├── index.js          # library 编译结果，导出库函数
├── cmd_name.js       # binary/cmd_name.vk 或 binary/cmd_name/main.vk 编译结果
└── test/
    └── index.js      # test/ 目录编译结果
```

### 依赖关系

- **Binary → Library**: 所有 binary 程序自动依赖 library 的输出
- **Test → Library**: 测试代码依赖 library 中的函数进行测试
- **包间依赖**: 构建系统自动处理包之间的依赖关系

## 示例

### 创建新项目

```bash
legion init my-app
cd my-app
legion build
```

### 添加依赖

在 `legion.json` 中添加依赖：

```json
{
  "dependencies": [
    "@valkyrie-language/math@1.0.0",
    "@valkyrie-language/io@latest"
  ]
}
```

### 编写 Library 代码

在 `library/example.valkyrie` 中：

```valkyrie
namespace mylib::example;

micro greet(name) {
    return "Hello, " + name + "!";
}

micro add(a, b) {
    return a + b;
}
```

### 编写 Binary 代码

**文件夹类型** - 在 `binary/myapp/main.vk` 中：

```valkyrie
namespace myapp;

using mylib::example;

micro main() {
    console.log(greet("World"));
    console.log("5 + 3 = " + add(5, 3));
}
```

**单独文件类型** - 在项目根目录 `tool.vk` 中：

```valkyrie
namespace tool;

using mylib::example;

micro main() {
    console.log("This is a standalone tool");
    console.log(greet("Tool User"));
}
```

## 构建流程

1. **配置加载** - 读取 `legion.yml` 配置文件
2. **依赖解析** - 安装和管理项目依赖
3. **源码编译** - 使用 valkyrie-bootstrap 编译 .vk 文件
4. **资源复制** - 复制非源码文件到构建目录
5. **构建信息** - 生成构建元数据

## 架构设计

Legion 采用模块化设计，每个模块负责特定的功能：

- **ConfigManager** - 处理项目配置和设置
- **ValkyrieCompiler** - 与 valkyrie-bootstrap 编译器交互
- **DependencyManager** - 管理项目依赖关系
- **WorkspaceManager** - 协调整个构建流程
- **ProjectTemplate** - 提供项目模板功能

## 未来计划

- [ ] 完整的 Valkyrie 实现
- [ ] 并行编译支持
- [ ] 增量编译优化
- [ ] 包发布和管理
- [ ] IDE 集成支持
- [ ] 测试框架集成

## 贡献

欢迎贡献代码！请确保：

1. 代码遵循 Valkyrie 语言规范
2. 添加适当的测试用例
3. 更新相关文档
4. 通过代码审查

## 许可证

MIT License - 详见 LICENSE 文件
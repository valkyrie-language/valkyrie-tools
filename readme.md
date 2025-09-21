# Valkyrie Language Monorepo

这是一个包含 Valkyrie 语言相关所有项目的 Monorepo。Valkyrie 是一种旨在提供高性能、安全和表达力的编程语言。

## 🚀 项目概述

本 Monorepo 旨在集中管理 Valkyrie 语言的各个组成部分，包括编译器、运行时、文档、工具等。每个子项目都专注于语言生态系统的一个特定方面。

## 📁 项目结构

```
valkyrie.valkyrie/
├── projects/                     # 包含所有 Valkyrie 语言相关的子项目
│   ├── valkyrie-bootstrap/       # Valkyrie 语言自举编译器项目
│   │   ├── bootstrap.js          # 自举编译脚本
│   │   ├── dist/                 # 编译产物（自举生成）
│   │   ├── library/              # Valkyrie 源代码
│   │   ├── package.json          # 项目配置
│   │   ├── readme.md             # 项目说明
│   │   └── test/                 # 测试文件
│   ├── valkyrie-document/        # Valkyrie 语言官方文档项目
│   │   ├── .vitepress/           # VitePress 配置
│   │   ├── docs/                 # 文档内容
│   │   ├── examples/             # 文档示例
│   │   ├── faq.md                # 常见问题
│   │   ├── guide/                # 指南
│   │   ├── index.md              # 文档首页
│   │   ├── language/             # 语言规范
│   │   ├── maintenance/          # 维护指南
│   │   ├── package.json          # 项目配置
│   │   └── readme.md             # 项目说明
├── readme.md                     # Monorepo 总览
└── ... (其他项目文件和目录)
```

## 💡 子项目介绍

- **`valkyrie-bootstrap`**: 这是 Valkyrie 语言的自举编译器项目。它使用 JavaScript 实现，能够将 Valkyrie 源代码编译为 JavaScript。该项目是 Valkyrie 语言实现自举的关键里程碑。
- **`valkyrie-document`**: 这是 Valkyrie 语言的官方文档项目。它包含了语言规范、使用指南、示例、常见问题解答以及维护说明等，旨在为用户和开发者提供全面的信息。

## 🚧 开发状态

当前版本是 Valkyrie 语言的早期开发阶段，自举编译器已成功完成。我们正在积极开发和完善语言的各个方面。

### 已实现
- ✅ Valkyrie 语言自举编译器 (valkyrie-bootstrap) 成功完成自举
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
- 🔄 更多核心库和运行时功能

## 🤝 贡献

欢迎贡献代码、报告问题或提出建议！请查阅各个子项目的 `readme.md` 以获取更详细的贡献指南。

## 📄 许可证

ISC License


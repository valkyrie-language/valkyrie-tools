# Valkyrie Git Lint

🚀 可配置的 Git 提交规范工具，支持 emoji 提交和 release 报告生成

## 特性

- ✨ **完全可配置**: 通过 `git-lint.json` 配置文件自定义所有规则
- 🎯 **双重验证**: 支持传统 conventional commits 和 emoji 提交格式
- 📊 **智能报告**: 自动生成 pnpm changeset 风格的 release 报告
- 🔧 **灵活优先级**: 可配置的 emoji 类型优先级和标签
- 📈 **统计分析**: 提供详细的提交统计信息
- 🛠️ **CLI 工具**: 完整的命令行接口
- 📦 **零依赖**: 不依赖外部库，轻量级设计

## 安装

```bash
npm install valkyrie-git-lint
# 或
pnpm add valkyrie-git-lint
# 或
yarn add valkyrie-git-lint
```

## 快速开始

### 1. 初始化配置

```bash
npx valkyrie-git-lint init
```

这将在当前目录创建 `git-lint.json` 配置文件。

### 2. 验证提交消息

```bash
# 验证传统格式
npx valkyrie-git-lint validate "feat: add new feature"

# 验证 emoji 格式
npx valkyrie-git-lint validate-emoji "✨ 添加新功能"
```

### 3. 生成 release 报告

```bash
# 生成指定版本的 release 报告
npx valkyrie-git-lint release v1.2.0 --from v1.1.0

# 生成完整 changelog
npx valkyrie-git-lint changelog
```

## 配置文件

`git-lint.json` 配置文件包含两个主要部分：

### commitlint 配置

```json
{
  "commitlint": {
    "extends": ["@commitlint/config-conventional"],
    "rules": {
      "type-enum": [2, "always", ["feat", "fix", "docs", "style", "refactor", "test"]],
      "type-empty": [2, "never"],
      "subject-empty": [2, "never"],
      "header-max-length": [2, "always", 72]
    },
    "prompt": {
      "questions": {
        "type": {
          "description": "选择你要提交的更改类型:",
          "enum": {
            "✨ feat": {
              "description": "新功能",
              "title": "Features",
              "emoji": "✨"
            }
          }
        }
      }
    }
  }
}
```

### releaseReport 配置

```json
{
  "releaseReport": {
    "emojiTypes": {
      "✨": {"name": "feature", "priority": 1, "label": "Stable Features"},
      "🔧": {"name": "fix", "priority": 4, "label": "Bug Fixes"},
      "📝": {"name": "docs", "priority": 6, "label": "Documentation Updates"}
    },
    "output": {
      "directory": "releases",
      "dateFormat": "zh-CN",
      "maxCommitsInPreview": 20
    }
  }
}
```

## CLI 命令

### 基本命令

```bash
# 显示帮助
valkyrie-git-lint --help

# 初始化配置文件
valkyrie-git-lint init

# 验证提交消息
valkyrie-git-lint validate "feat: add feature"
valkyrie-git-lint validate-emoji "✨ 添加功能"

# 显示允许的类型
valkyrie-git-lint types

# 显示当前配置
valkyrie-git-lint config
```

### 报告生成

```bash
# 生成 release 报告
valkyrie-git-lint release v1.2.0 --from v1.1.0 --to HEAD

# 生成完整 changelog
valkyrie-git-lint changelog --output CHANGELOG.md

# 显示提交统计
valkyrie-git-lint stats --from v1.1.0
```

### 选项

- `--config <path>`: 指定配置文件路径
- `--from <tag>`: 起始 tag
- `--to <tag>`: 结束 tag (默认: HEAD)
- `--output <file>`: 输出文件名

## API 使用

### 基本用法

```javascript
const ValkyrieGitLint = require('valkyrie-git-lint');

const gitLint = new ValkyrieGitLint('./git-lint.json');

// 验证提交消息
const result = gitLint.validateEmojiCommitMessage('✨ 添加新功能');
console.log(result.valid); // true/false
console.log(result.errors); // 错误信息数组

// 生成 release 报告
const report = gitLint.generateReleaseReport('v1.2.0', 'v1.1.0');
console.log(report);

// 获取统计信息
const stats = gitLint.getCommitStatistics('v1.1.0');
console.log(stats);
```

### 高级用法

```javascript
const { CommitLint, ReleaseReportGenerator, ConfigLoader } = require('valkyrie-git-lint');

// 单独使用各个模块
const commitLint = new CommitLint('./git-lint.json');
const releaseGenerator = new ReleaseReportGenerator('./git-lint.json');
const configLoader = new ConfigLoader();

// 获取配置
const config = configLoader.loadConfig('./git-lint.json');

// 获取类型选择列表
const choices = commitLint.getTypeChoices();

// 生成 commitlint 配置
const commitlintConfig = commitLint.generateCommitlintConfig();
```

## 配置说明

### Emoji 类型配置

每个 emoji 类型包含以下属性：

- `name`: 内部名称
- `priority`: 优先级（数字越小优先级越高，负数表示不在报告中显示）
- `label`: 在报告中显示的标签

### 规则配置

支持 commitlint 标准规则格式：

- `[level, applicable, value]`
- `level`: 0=禁用, 1=警告, 2=错误
- `applicable`: "always" 或 "never"
- `value`: 规则值

## 与原始脚本的对比

| 功能 | 原始脚本 | Valkyrie Git Lint |
|------|----------|-------------------|
| 配置方式 | 硬编码 | JSON 配置文件 |
| Emoji 类型 | 固定 | 完全可配置 |
| 优先级 | 固定 | 可配置 |
| 标签 | 固定 | 可配置 |
| CLI 接口 | 基础 | 完整功能 |
| API 接口 | 无 | 完整 API |
| 验证功能 | 无 | 双重验证 |
| 统计功能 | 基础 | 详细统计 |

## 开发

### 项目结构

```
valkyrie-git-lint/
├── src/
│   ├── index.js              # 主入口
│   ├── config-loader.js      # 配置加载器
│   ├── commit-lint.js        # 提交验证
│   └── release-report.js     # 报告生成
├── bin/
│   └── valkyrie-git-lint.js  # CLI 入口
├── tests/
│   └── test.js               # 测试文件
├── git-lint.json             # 配置文件 schema
├── git-lint.default.json     # 默认配置
├── package.json
└── README.md
```

### 运行测试

```bash
npm test
```

### 代码检查

```bash
npm run lint
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 更新日志

### v1.0.0

- ✨ 初始版本发布
- 🔧 完整的配置系统
- 📊 Release 报告生成
- 🛠️ CLI 工具
- 📝 完整文档
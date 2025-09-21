# Release Report Generator

一个基于 emoji commit 规范的自动化 release 报告生成工具。

## 功能特点

- 🎯 **智能分类**: 根据 emoji 类型自动分类提交记录
- 📊 **优先级排序**: 按照 feature → experiment → fix → other 的顺序排列
- 📝 **Markdown 格式**: 生成标准的 Markdown 格式报告
- 🔗 **自动链接**: 自动生成 commit 链接
- 📅 **日期自动**: 自动添加发布日期

## Emoji 类型映射

| Emoji | 类型         | 优先级 | 中文标签  |
|-------|------------|-----|-------|
| ✨     | feature    | 1   | 新功能   |
| 🔮    | experiment | 2   | 实验性功能 |
| 🔧    | fix        | 3   | 修复    |
| 📝    | docs       | 4   | 文档    |
| 🎨    | style      | 4   | 样式    |
| ☢️    | refactor   | 4   | 重构    |
| 🧪    | test       | 4   | 测试    |
| 🔨    | config     | 4   | 配置    |
| ⚡️    | perf       | 4   | 性能    |
| 🚀    | release    | 4   | 发布    |
| 🔖    | tag        | 4   | 标签    |
| 🚦    | ci         | 4   | CI/CD |
| 📦    | build      | 4   | 构建    |
| ⏪     | revert     | 4   | 回滚    |
| 💡    | idea       | 4   | 想法    |
| 🧨    | delete     | 4   | 删除    |
| ✅     | complete   | 4   | 完成    |
| 🔀    | branch     | 4   | 分支    |

## 使用方法

### 基本用法

```bash
# 生成当前版本的 release 报告
node scripts/generate-release-report.js

# 指定版本号
node scripts/generate-release-report.js v1.2.0

# 通过 npm 脚本运行
npm run release:report -- v1.2.0
```

### 高级用法

```bash
# 指定起始 tag
node scripts/generate-release-report.js v1.2.0 --from v1.1.0

# 指定起始和结束 tag
node scripts/generate-release-report.js v1.2.0 --from v1.1.0 --to v1.2.0

# 自定义输出文件名
node scripts/generate-release-report.js v1.2.0 --output my-release.md

# 组合使用
node scripts/generate-release-report.js v1.2.0 --from v1.1.0 --output releases/v1.2.0-release.md
```

### 查看帮助

```bash
node scripts/generate-release-report.js --help
```

## 输出示例

生成的报告会按照以下格式组织：

```markdown
# 🚀 Release v1.2.0

发布日期: 2025/9/21 20:53:07

## ✨ 新功能 (5)

- 添加用户登录功能 ([abc1234](../../commit/abc1234))
- 实现搜索功能 ([def5678](../../commit/def5678))
- 添加深色模式 ([ghi9012](../../commit/ghi9012))

## 🔮 实验性功能 (2)

- 实验性 AI 推荐系统 ([jkl3456](../../commit/jkl3456))
- 测试新的算法 ([mno7890](../../commit/mno7890))

## 🔧 修复 (8)

- 修复首页加载慢的问题 ([pqr1234](../../commit/pqr1234))
- 解决移动端显示异常 ([stu5678](../../commit/stu5678))

## 📝 文档 (3)

- 更新 API 文档 ([vwx9012](../../commit/vwx9012))
- 添加使用指南 ([yza3456](../../commit/yza3456))
```

## 集成到工作流

### 在 package.json 中添加脚本

```json
{
  "scripts": {
    "release:report": "node scripts/generate-release-report.js",
    "release:latest": "node scripts/generate-release-report.js $(date +v%Y.%m.%d)"
  }
}
```

### 自动化发布流程

```bash
# 1. 生成 release 报告
npm run release:report -- v1.2.0 --from v1.1.0

# 2. 查看生成的报告
cat releases/RELEASE-2025-09-21.md

# 3. 添加到 git
git add releases/
git commit -m "🚀 release: 添加 v1.2.0 release 报告"
```

## 注意事项

1. **Commit 规范**: 确保所有 commit 都遵循 emoji + 空格 + 描述的格式
2. **Tag 管理**: 使用 git tag 来标记版本，便于生成版本间的差异报告
3. **文件路径**: 报告默认保存在 `releases/` 目录下

## 扩展功能

可以通过修改 `EMOJI_TYPES` 常量来添加新的 emoji 类型或调整优先级：

```javascript
const EMOJI_TYPES = {
    '🌟': {name: 'highlight', priority: 1, label: '亮点功能'},
    // 添加更多自定义类型...
};
```
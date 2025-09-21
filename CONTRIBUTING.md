# 贡献指南

## 欢迎贡献

感谢您对本项目的关注！本指南将帮助您了解如何为项目做出贡献。

## Commit Message 规范

所有提交必须遵循 emoji commit 规范，以确保提交历史的清晰和一致性。

### 格式要求
```
[emoji] [空格] [commit消息]
```

### Emoji 类型

#### 功能相关
- ✨ `feat`: 新功能
- 🔧 `fix`: 修复bug
- 📝 `docs`: 文档更新
- 🎨 `style`: 代码格式调整
- 🔁 `refactor`: 重构代码
- 🧪 `test`: 测试相关
- 🔨 `config`: 配置文件修改
- ⚡️ `perf`: 性能优化

#### 流程相关
- 🚀 `release`: 发布版本
- 🔖 `tag`: 标签相关
- 🚦 `ci`: CI/CD相关
- 📦 `build`: 构建相关
- ⏪ `revert`: 回滚操作

#### 其他
- 💡 `idea`: 新想法
- 🗑️ `delete`: 删除文件
- 📦 `add`: 添加文件
- ✅ `complete`: 完成任务
- 🔀 `branch`: 分支操作

### 示例
```
✨ 添加用户登录功能
🔧 修复首页加载缓慢的问题
📝 更新API文档
🎨 格式化代码风格
🔁 重构用户模块
🧪 添加单元测试
🔨 修改数据库配置
⚡️ 优化查询性能
```

## 开发环境设置

### 1. 安装依赖
```bash
npm install
```

### 2. 配置 commit 工具
运行自动化设置脚本：

**Linux/macOS:**
```bash
./scripts/setup-emoji-commit.sh
```

**Windows:**
```bash
scripts\setup-emoji-commit.bat
```

### 3. 提交代码
使用交互式提交工具：
```bash
npm run commit
# 或
git-cz
```

或直接提交（确保遵循emoji规范）：
```bash
git commit -m "✨ 添加新功能"
```

### 4. 分支管理
**禁止使用 merge，必须使用 rebase：**
```bash
# 拉取最新代码并 rebase
git pull --rebase origin main

# 合并特性分支
git checkout main
git pull --rebase origin main
git rebase feature-branch
```

## 代码审查

所有 Pull Request 都需要经过代码审查，请确保：
- 所有 commit 都遵循 emoji commit 规范
- 代码已经过测试
- 文档已更新（如需要）
- 遵循项目的编码规范

## 问题报告

如果您发现 bug 或有功能建议，请创建 Issue 并包含：
- 清晰的标题
- 详细的问题描述
- 重现步骤
- 期望的行为
- 实际行为
- 环境信息


#!/bin/bash

# Emoji Commit 设置脚本
# 自动安装和配置emoji commit工具

echo "🚀 正在设置 Emoji Commit 规范..."

# 检查Node.js是否安装
if ! command -v node &> /dev/null; then
    echo "❌ 错误: 请先安装 Node.js"
    exit 1
fi

# 检查npm是否安装
if ! command -v npm &> /dev/null; then
    echo "❌ 错误: 请先安装 npm"
    exit 1
fi

# 安装项目依赖
echo "📦 安装项目依赖..."
npm install

# 全局安装commit工具
echo "🔧 安装全局commit工具..."
npm install -g commitizen @commitlint/cli

# 设置git hook（如果git仓库存在）
if [ -d ".git" ]; then
    echo "🔗 设置git hook..."
    cp .git/hooks/commit-msg.sample .git/hooks/commit-msg 2>/dev/null || touch .git/hooks/commit-msg
    
    # 写入hook内容
    cat > .git/hooks/commit-msg << 'EOF'
#!/bin/sh

# Emoji Commit Git Hook
# 强制所有commit消息以emoji+空格开头

# 获取commit消息
commit_msg=$(cat "$1")

# 检查是否以emoji+空格开头
if ! echo "$commit_msg" | grep -qE "^[✨🐛📝🎨♻️🧪🔧⚡️🚀🔖🚦📦🔄⏪💡🗑️📦✅🔀] "; then
    echo "错误: commit消息必须以emoji+空格开头"
    echo ""
    echo "示例:"
    echo "  ✨ 添加新功能"
    echo "  🐛 修复首页加载问题"
    echo "  📝 更新API文档"
    echo ""
    echo "支持的emoji类型:"
    echo "  ✨ feat: 新功能"
    echo "  🐛 fix: 修复bug"
    echo "  📝 docs: 文档更新"
    echo "  🎨 style: 代码格式调整"
    echo "  ♻️ refactor: 重构代码"
    echo "  🧪 test: 测试相关"
    echo "  🔧 config: 配置文件修改"
    echo "  ⚡️ perf: 性能优化"
    echo "  🚀 release: 发布版本"
    echo "  🔖 tag: 标签相关"
    echo "  🚦 ci: CI/CD相关"
    echo "  📦 build: 构建相关"
    echo "  🔄 merge: 合并分支"
    echo "  ⏪ revert: 回滚操作"
    echo "  💡 idea: 新想法"
    echo "  🗑️ delete: 删除文件"
    echo "  📦 add: 添加文件"
    echo "  ✅ complete: 完成任务"
    echo "  🔀 branch: 分支操作"
    echo ""
    echo "请使用 'npm run commit' 进行交互式提交，或手动添加emoji前缀。"
    exit 1
fi

# 检查消息长度（不超过72个字符）
if [ ${#commit_msg} -gt 72 ]; then
    echo "警告: commit消息建议不超过72个字符"
fi

exit 0
EOF
    
    # 添加执行权限
    chmod +x .git/hooks/commit-msg
    echo "✅ Git hook 设置完成"
else
    echo "⚠️  警告: 未找到.git目录，跳过git hook设置"
fi

echo ""
echo "✅ Emoji Commit 规范设置完成！"
echo ""
echo "📖 使用方法:"
echo "  1. 交互式提交: npm run commit"
echo "  2. 直接提交: git commit -m '✨ 添加新功能'"
echo "  3. 查看文档: cat CONTRIBUTING.md"
echo ""
echo "🎯 支持的emoji类型:"
echo "  ✨ feat: 新功能"
echo "  🐛 fix: 修复bug"
echo "  📝 docs: 文档更新"
echo "  🎨 style: 代码格式调整"
echo "  ♻️ refactor: 重构代码"
echo "  🧪 test: 测试相关"
echo "  🔧 config: 配置文件修改"
echo "  ⚡️ perf: 性能优化"
echo "  🚀 release: 发布版本"
echo "  🔖 tag: 标签相关"
echo "  🚦 ci: CI/CD相关"
echo "  📦 build: 构建相关"
echo "  🔄 merge: 合并分支"
echo "  ⏪ revert: 回滚操作"
echo "  💡 idea: 新想法"
echo "  🗑️ delete: 删除文件"
echo "  📦 add: 添加文件"
echo "  ✅ complete: 完成任务"
echo "  🔀 branch: 分支操作"
echo ""
echo "🎉 现在开始使用emoji commit吧！"
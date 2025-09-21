@echo off
chcp 65001 >nul
echo 🚀 正在设置 Emoji Commit 规范...

REM 检查Node.js是否安装
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ 错误: 请先安装 Node.js
    pause
    exit /b 1
)

REM 检查npm是否安装
npm --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ 错误: 请先安装 npm
    pause
    exit /b 1
)

REM 安装项目依赖
echo 📦 安装项目依赖...
npm install
if %errorlevel% neq 0 (
    echo ❌ 依赖安装失败
    pause
    exit /b 1
)

REM 全局安装commit工具
echo 🔧 安装全局commit工具...
npm install -g commitizen @commitlint/cli
if %errorlevel% neq 0 (
    echo ❌ 全局工具安装失败
    pause
    exit /b 1
)

REM 设置git hook（如果git仓库存在）
if exist .git (
    echo 🔗 设置git hook...
    
    REM 创建commit-msg hook文件
    echo #!/bin/sh > .git\hooks\commit-msg
    echo. >> .git\hooks\commit-msg
    echo # Emoji Commit Git Hook >> .git\hooks\commit-msg
    echo # 强制所有commit消息以emoji+空格开头 >> .git\hooks\commit-msg
    echo. >> .git\hooks\commit-msg
    echo # 获取commit消息 >> .git\hooks\commit-msg
    echo commit_msg=$(cat "$1") >> .git\hooks\commit-msg
    echo. >> .git\hooks\commit-msg
    echo # 检查是否以emoji+空格开头 >> .git\hooks\commit-msg
    echo if ! echo "$commit_msg" ^| grep -qE "^[✨🐛📝🎨♻️🧪🔧⚡️🚀🔖🚦📦🔄⏪💡🗑️📦✅🔀] "; then >> .git\hooks\commit-msg
    echo     echo "错误: commit消息必须以emoji+空格开头" >> .git\hooks\commit-msg
    echo     echo "" >> .git\hooks\commit-msg
    echo     echo "示例:" >> .git\hooks\commit-msg
    echo     echo "  ✨ 添加新功能" >> .git\hooks\commit-msg
    echo     echo "  🐛 修复首页加载问题" >> .git\hooks\commit-msg
    echo     echo "  📝 更新API文档" >> .git\hooks\commit-msg
    echo     echo "" >> .git\hooks\commit-msg
    echo     echo "支持的emoji类型:" >> .git\hooks\commit-msg
    echo     echo "  ✨ feat: 新功能" >> .git\hooks\commit-msg
    echo     echo "  🐛 fix: 修复bug" >> .git\hooks\commit-msg
    echo     echo "  📝 docs: 文档更新" >> .git\hooks\commit-msg
    echo     echo "  🎨 style: 代码格式调整" >> .git\hooks\commit-msg
    echo     echo "  ♻️ refactor: 重构代码" >> .git\hooks\commit-msg
    echo     echo "  🧪 test: 测试相关" >> .git\hooks\commit-msg
    echo     echo "  🔧 config: 配置文件修改" >> .git\hooks\commit-msg
    echo     echo "  ⚡️ perf: 性能优化" >> .git\hooks\commit-msg
    echo     echo "  🚀 release: 发布版本" >> .git\hooks\commit-msg
    echo     echo "  🔖 tag: 标签相关" >> .git\hooks\commit-msg
    echo     echo "  🚦 ci: CI/CD相关" >> .git\hooks\commit-msg
    echo     echo "  📦 build: 构建相关" >> .git\hooks\commit-msg
    echo     echo "  🔄 merge: 合并分支" >> .git\hooks\commit-msg
    echo     echo "  ⏪ revert: 回滚操作" >> .git\hooks\commit-msg
    echo     echo "  💡 idea: 新想法" >> .git\hooks\commit-msg
    echo     echo "  🗑️ delete: 删除文件" >> .git\hooks\commit-msg
    echo     echo "  📦 add: 添加文件" >> .git\hooks\commit-msg
    echo     echo "  ✅ complete: 完成任务" >> .git\hooks\commit-msg
    echo     echo "  🔀 branch: 分支操作" >> .git\hooks\commit-msg
    echo     echo "" >> .git\hooks\commit-msg
    echo     echo "请使用 'npm run commit' 进行交互式提交，或手动添加emoji前缀。" >> .git\hooks\commit-msg
    echo     exit 1 >> .git\hooks\commit-msg
    echo fi >> .git\hooks\commit-msg
    echo. >> .git\hooks\commit-msg
    echo # 检查消息长度（不超过72个字符） >> .git\hooks\commit-msg
    echo if [ ${#commit_msg} -gt 72 ]; then >> .git\hooks\commit-msg
    echo     echo "警告: commit消息建议不超过72个字符" >> .git\hooks\commit-msg
    echo fi >> .git\hooks\commit-msg
    echo. >> .git\hooks\commit-msg
    echo exit 0 >> .git\hooks\commit-msg
    
    echo ✅ Git hook 设置完成
) else (
    echo ⚠️  警告: 未找到.git目录，跳过git hook设置
)

echo.
echo ✅ Emoji Commit 规范设置完成！
echo.
echo 📖 使用方法:
echo   1. 交互式提交: npm run commit
echo   2. 直接提交: git commit -m '✨ 添加新功能'
echo   3. 查看文档: type CONTRIBUTING.md
echo.
echo 🎯 支持的emoji类型:
echo   ✨ feat: 新功能
echo   🐛 fix: 修复bug
echo   📝 docs: 文档更新
echo   🎨 style: 代码格式调整
echo   ♻️ refactor: 重构代码
echo   🧪 test: 测试相关
echo   🔧 config: 配置文件修改
echo   ⚡️ perf: 性能优化
echo   🚀 release: 发布版本
echo   🔖 tag: 标签相关
echo   🚦 ci: CI/CD相关
echo   📦 build: 构建相关
echo   🔄 merge: 合并分支
echo   ⏪ revert: 回滚操作
echo   💡 idea: 新想法
echo   🗑️ delete: 删除文件
echo   📦 add: 添加文件
echo   ✅ complete: 完成任务
echo   🔀 branch: 分支操作
echo.
echo 🎉 现在开始使用emoji commit吧！
echo.
pause
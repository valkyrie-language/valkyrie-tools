#!/bin/bash

# Bootstrap 构建脚本
echo "🔧 正在构建 bootstrap 脚本..."

# 创建输出目录
mkdir -p dist

# 复制脚本文件
cp scripts/generate-release-report.js dist/

# 如果有其他需要打包的文件，在这里添加
cp package.json dist/ 2>/dev/null || true
cp package-lock.json dist/ 2>/dev/null || true
cp README.md dist/ 2>/dev/null || true

echo "✅ Bootstrap 构建完成！"
echo "📦 输出目录: dist/"

# 列出构建的文件
echo "📁 构建文件:"
ls -la dist/
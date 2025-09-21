const ReleaseReportGenerator = require('./projects/valkyrie-git-lint/dist/release-report.js').ReleaseReportGenerator;

// 使用完整的配置对象
const config = {
    emojiConfig: {
        ':sparkles:': { name: 'feat', label: '✨ Features', description: '新功能', priority: 1 },
        '✨': { name: 'feat', label: '✨ Features', description: '新功能', priority: 1 },
        ':bug:': { name: 'fix', label: '🐛 Bug Fixes', description: 'Bug 修复', priority: 2 },
        '🐛': { name: 'fix', label: '🐛 Bug Fixes', description: 'Bug 修复', priority: 2 },
        '🧨': { name: 'fix', label: '🐛 Bug Fixes', description: 'Bug 修复', priority: 2 },
        ':memo:': { name: 'docs', label: '📝 Documentation', description: '文档变更', priority: 3 },
        '📝': { name: 'docs', label: '📝 Documentation', description: '文档变更', priority: 3 },
        ':lipstick:': { name: 'style', label: '💄 Styles', description: '代码格式 (不影响代码运行的变动)', priority: 4 },
        '💄': { name: 'style', label: '💄 Styles', description: '代码格式 (不影响代码运行的变动)', priority: 4 },
        ':recycle:': { name: 'refactor', label: '♻️ Code Refactoring', description: '代码重构 (不包括 bug 修复、功能新增)', priority: 5 },
        '♻️': { name: 'refactor', label: '♻️ Code Refactoring', description: '代码重构 (不包括 bug 修复、功能新增)', priority: 5 },
        ':zap:': { name: 'perf', label: '⚡ Performance Improvements', description: '性能优化', priority: 6 },
        '⚡': { name: 'perf', label: '⚡ Performance Improvements', description: '性能优化', priority: 6 },
        ':white_check_mark:': { name: 'test', label: '✅ Tests', description: '测试', priority: 7 },
        '✅': { name: 'test', label: '✅ Tests', description: '测试', priority: 7 },
        ':wrench:': { name: 'build', label: '🔧 Build System', description: '构建过程或辅助工具的变动', priority: 8 },
        '🔧': { name: 'build', label: '🔧 Build System', description: '构建过程或辅助工具的变动', priority: 8 },
        '🔨': { name: 'build', label: '🔧 Build System', description: '构建过程或辅助工具的变动', priority: 8 },
        ':construction_worker:': { name: 'ci', label: '👷 Continuous Integration', description: 'CI 配置、脚本的变更', priority: 9 },
        '👷': { name: 'ci', label: '👷 Continuous Integration', description: 'CI 配置、脚本的变更', priority: 9 },
        ':rocket:': { name: 'chore', label: '🚀 Chores', description: '其他不修改 src 或 test 目录的提交', priority: 10 },
        '🚀': { name: 'chore', label: '🚀 Chores', description: '其他不修改 src 或 test 目录的提交', priority: 10 },
        ':rewind:': { name: 'revert', label: '⏪ Reverts', description: '回滚 commit', priority: 11 },
        '⏪': { name: 'revert', label: '⏪ Reverts', description: '回滚 commit', priority: 11 },
        ':package:': { name: 'dep', label: '📦 Dependency Updates', description: '依赖更新', priority: 12 },
        '📦': { name: 'dep', label: '📦 Dependency Updates', description: '依赖更新', priority: 12 }
    },
    releaseReport: {
        output: {
            directory: 'releases',
            dateFormat: 'zh-CN',
            maxCommitsInPreview: 1000,
            filename: 'CHANGELOG.md'
        }
    }
};

const generator = new ReleaseReportGenerator(config);
console.log('Starting test...');
const changelog = generator.generateChangelog();
console.log('Test completed.');
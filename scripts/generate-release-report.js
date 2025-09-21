#!/usr/bin/env node

/**
 * Release Report Generator
 * 根据 emoji commit 规范自动生成 release 报告
 *
 * 优先级顺序：
 * 1. ✨ feature (新功能)
 * 2. 🔮 experiment (实验性功能)
 * 3. 🔧 fix (修复bug)
 * 4. 其他类型
 */

const {execSync} = require('child_process');
const fs = require('fs');
const path = require('path');

// Emoji 类型映射和优先级
const EMOJI_TYPES = {
    '✨': {name: 'feature', priority: 1, label: '稳定特性'},
    '🔮': {name: 'experiment', priority: 2, label: '实验特性'},
    '🔧': {name: 'fix', priority: 3, label: 'Bug 修复'},
    '📝': {name: 'documentation', priority: 4, label: '文档更新'},
    '🎨': {name: 'style', priority: 4, label: '样式'},
    '☢️': {name: 'refactor', priority: 4, label: '重构'},
    '🧪': {name: 'test', priority: 4, label: '测试'},
    '🔨': {name: 'config', priority: 4, label: '配置'},
    '⚡️': {name: 'perf', priority: 4, label: '性能'},
    '🚀': {name: 'release', priority: 4, label: '发布'},
    '🔖': {name: 'tag', priority: 4, label: '标签'},
    '🚦': {name: 'ci', priority: 4, label: 'CI/CD'},
    '📦': {name: 'build', priority: 4, label: '构建'},
    '⏪': {name: 'revert', priority: 4, label: '回滚'},
    '💡': {name: 'idea', priority: 4, label: '想法'},
    '🧨': {name: 'delete', priority: 4, label: '删除'},
    '✅': {name: 'complete', priority: 4, label: '完成'},
    '🔀': {name: 'branch', priority: 4, label: '分支'}
};

class ReleaseReportGenerator {
    constructor() {
        this.commits = [];
        this.groupedCommits = {};
    }

    /**
     * 获取两个 tag 之间的提交记录
     */
    getCommitsBetweenTags(fromTag, toTag = 'HEAD') {
        try {
            const command = fromTag
                ? `git log ${fromTag}..${toTag} --oneline`
                : `git log --oneline -20`; // 如果没有 fromTag，获取最近20条

            const output = execSync(command, {encoding: 'utf8'});
            return output.trim().split('\n').filter(line => line.length > 0);
        } catch (error) {
            console.error('获取提交记录失败:', error.message);
            return [];
        }
    }

    /**
     * 解析提交记录
     */
    parseCommit(line) {
        const match = line.match(/^([a-f0-9]+)\s+([✨🔧📝🎨☢️🧪🔨⚡️🚀🔖🚦📦⏪💡🧨✅🔀🔮])\s+(.+)$/);
        if (!match) {
            return null;
        }

        const [, hash, emoji, message] = match;
        const type = EMOJI_TYPES[emoji] || {name: 'other', priority: 5, label: '其他'};

        return {
            hash: hash.slice(0, 7), // 只取前7位
            emoji,
            type: type.name,
            priority: type.priority,
            label: type.label,
            message: message.trim()
        };
    }

    /**
     * 按类型分组提交
     */
    groupCommitsByType(commits) {
        const groups = {};

        commits.forEach(commit => {
            if (!commit) return;

            const type = commit.type;
            if (!groups[type]) {
                groups[type] = [];
            }
            groups[type].push(commit);
        });

        return groups;
    }

    /**
     * 生成 Markdown 格式的报告
     */
    generateMarkdownReport(version, fromTag, toTag = 'HEAD') {
        const commitLines = this.getCommitsBetweenTags(fromTag, toTag);
        const parsedCommits = commitLines.map(line => this.parseCommit(line)).filter(Boolean);

        if (parsedCommits.length === 0) {
            return '没有找到符合规范的提交记录。';
        }

        const groupedCommits = this.groupCommitsByType(parsedCommits);

        // 按优先级排序的类型
        const sortedTypes = Object.keys(groupedCommits).sort((a, b) => {
            const priorityA = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === a)?.name] || {priority: 5};
            const priorityB = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === b)?.name] || {priority: 5};
            return priorityA.priority - priorityB.priority;
        });

        let report = `# 🚀 Release ${version}\n\n`;
        report += `发布日期: ${new Date().toLocaleString('zh-CN')}\n\n`;

        sortedTypes.forEach(type => {
            const commits = groupedCommits[type];
            if (commits.length === 0) return;

            const emoji = commits[0].emoji;
            const label = EMOJI_TYPES[emoji]?.label || type;

            report += `## ${emoji} ${label} (${commits.length})\n\n`;
            commits.forEach(commit => {
                report += `- ${commit.message} ([${commit.hash}](../../commit/${commit.hash}))\n`;
            });
            report += '\n';
        });

        return report;
    }

    /**
     * 保存报告到文件
     */
    saveReport(content, filename = null) {
        const timestamp = new Date().toISOString().slice(0, 10);
        const defaultFilename = `RELEASE-${timestamp}.md`;
        const outputFile = filename || defaultFilename;
        const outputPath = path.join(process.cwd(), 'releases', outputFile);

        // 确保 releases 目录存在
        const releasesDir = path.dirname(outputPath);
        if (!fs.existsSync(releasesDir)) {
            fs.mkdirSync(releasesDir, {recursive: true});
        }

        fs.writeFileSync(outputPath, content, 'utf8');
        return outputPath;
    }
}

// CLI 处理
function main() {
    const args = process.argv.slice(2);

    if (args.includes('--help') || args.includes('-h')) {
        console.log(`
🚀 Release Report Generator

使用方法:
  node generate-release-report.js [版本号] [选项]

参数:
  版本号          发布的版本号 (默认: 当前日期)
  
选项:
  --from <tag>    起始 tag (默认: 最近20条提交)
  --to <tag>      结束 tag (默认: HEAD)
  --output <file> 输出文件名 (默认: RELEASE-YYYY-MM-DD.md)
  --help, -h      显示帮助信息

示例:
  node generate-release-report.js v1.2.0
  node generate-release-report.js v1.2.0 --from v1.1.0
  node generate-release-report.js v1.2.0 --from v1.1.0 --output my-release.md
`);
        return;
    }

    const version = args.find((arg, index) => index === 0 && !arg.startsWith('--')) || `v${new Date().toISOString().slice(0, 10)}`;
    const fromTag = args[args.indexOf('--from') + 1];
    const toTag = args[args.indexOf('--to') + 1] || 'HEAD';
    const outputFile = args[args.indexOf('--output') + 1];

    const generator = new ReleaseReportGenerator();

    console.log(`🔄 正在生成 release 报告...`);
    console.log(`📋 版本: ${version}`);
    if (fromTag) console.log(`📍 起始: ${fromTag}`);
    console.log(`📍 结束: ${toTag}`);
    console.log('');

    try {
        const report = generator.generateMarkdownReport(version, fromTag, toTag);
        const outputPath = generator.saveReport(report, outputFile);

        console.log('✅ Release 报告生成成功!');
        console.log(`📄 文件路径: ${outputPath}`);
        console.log('\n📊 报告预览:');
        console.log(report.split('\n').slice(0, 20).join('\n'));
        if (report.split('\n').length > 20) {
            console.log('...');
        }
    } catch (error) {
        console.error('❌ 生成报告失败:', error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = ReleaseReportGenerator;
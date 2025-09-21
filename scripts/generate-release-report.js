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
    '✨': {name: 'feature', priority: 1, label: 'Stable Features'},
    '🔮': {name: 'experiment', priority: 2, label: 'Experimental Features'},
    '☢️': {name: 'breaking', priority: 3, label: 'Breaking Changes'},
    '🔧': {name: 'fix', priority: 4, label: 'Bug Fixes'},
    '⚡️': {name: 'perf', priority: 5, label: 'Performance Improvements'},
    '📝': {name: 'docs', priority: 6, label: 'Documentation Updates'},
    '🎨': {name: 'style', priority: 9, label: 'Style Improvements'},
    '🧪': {name: 'test', priority: 7, label: 'Tests'},
    '🔨': {name: 'refactor', priority: 2, label: 'Refactoring'},
    '🚦': {name: 'ci', priority: 8, label: 'CI/CD'},
    '📦': {name: 'deps', priority: 3, label: 'Dependencies'},
    '⏪': {name: 'revert', priority: 3, label: '回滚'},
    '💡': {name: 'idea', priority: 9, label: '想法'},
    '🧨': {name: 'delete', priority: 9, label: '删除'},
    '✅': {name: 'done', priority: 9, label: '完成'},
    '🔀': {name: 'merge', priority: -1, label: '合并'},
    '🚀': {name: 'release', priority: -1, label: '发布'},
    '🔖': {name: 'tag', priority: -1, label: '标签'},
};

class ReleaseReportGenerator {
    constructor() {
        this.commits = [];
        this.groupedCommits = {};
    }

    /**
     * 获取两个 tag 之间的提交记录，包含作者信息
     */
    getCommitsBetweenTags(fromTag, toTag = 'HEAD') {
        try {
            // 先获取基本的提交信息
            const logCommand = fromTag
                ? `git log ${fromTag}..${toTag} --oneline`
                : `git log --oneline -20`;
            
            const logOutput = execSync(logCommand, {encoding: 'utf8'});
            const commits = logOutput.trim().split('\n').filter(line => line.length > 0);

            // 为每个提交获取作者信息
            return commits.map(line => {
                const hash = line.split(' ')[0];
                try {
                    const author = execSync(`git log -1 --format=%an ${hash}`, {encoding: 'utf8'}).trim();
                    return `${line}|${author}`;
                } catch (error) {
                    return `${line}|Unknown`;
                }
            });
        } catch (error) {
            console.error('获取提交记录失败:', error.message);
            return [];
        }
    }

    /**
     * 获取所有历史提交记录，包含作者信息
     */
    getAllCommits() {
        try {
            const logCommand = 'git log --oneline';
            const logOutput = execSync(logCommand, {encoding: 'utf8'});
            const commits = logOutput.trim().split('\n').filter(line => line.length > 0);

            // 为每个提交获取作者信息
            return commits.map(line => {
                const hash = line.split(' ')[0];
                try {
                    const author = execSync(`git log -1 --format=%an ${hash}`, {encoding: 'utf8'}).trim();
                    return `${line}|${author}`;
                } catch (error) {
                    return `${line}|Unknown`;
                }
            });
        } catch (error) {
            console.error('获取所有提交记录失败:', error.message);
            return [];
        }
    }

    /**
     * 获取 tag 的创建时间
     */
    getTagDate(tag) {
        try {
            if (!tag || tag === 'HEAD') {
                return new Date();
            }
            const timestamp = execSync(`git log -1 --format=%ai ${tag}`, {encoding: 'utf8'}).trim();
            return new Date(timestamp);
        } catch (error) {
            console.warn(`无法获取 tag ${tag} 的时间，使用当前时间`);
            return new Date();
        }
    }

    /**
     * 解析提交记录
     */
    parseCommit(line) {
        // 格式: hash emoji message | author
        const parts = line.split('|');
        if (parts.length !== 2) {
            return null;
        }

        const [commitInfo, author] = parts;
        
        // 提取 hash 和消息部分
        const commitMatch = commitInfo.match(/^([a-f0-9]+)\s+(.+)$/);
        if (!commitMatch) {
            return null;
        }

        const [, hash, message] = commitMatch;
        
        // 提取 emoji 和消息内容
        const emojiMatch = message.match(/^([✨🔧📝🎨☢️🧪🔨⚡️🚀🔖🚦📦⏪💡🧨✅🔀🔮])\s+(.+)$/);
        if (!emojiMatch) {
            return null;
        }

        const [, emoji, msgContent] = emojiMatch;
        const type = EMOJI_TYPES[emoji] || {name: 'other', priority: 5, label: '其他'};

        return {
            hash: hash.slice(0, 7), // 只取前7位
            emoji,
            type: type.name,
            priority: type.priority,
            label: type.label,
            message: msgContent.trim(),
            author: author.trim()
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
     * 生成 pnpm changeset 风格的 Markdown 报告
     */
    generateMarkdownReport(version, fromTag, toTag = 'HEAD') {
        const commitLines = this.getCommitsBetweenTags(fromTag, toTag);
        const parsedCommits = commitLines.map(line => this.parseCommit(line)).filter(Boolean);

        if (parsedCommits.length === 0) {
            return '## Unreleased\n\n没有找到符合规范的提交记录。\n';
        }

        const groupedCommits = this.groupCommitsByType(parsedCommits);

        // 按优先级排序的类型，过滤掉 priority < 0 的类型
        const sortedTypes = Object.keys(groupedCommits)
            .filter(type => {
                const typeInfo = Object.values(EMOJI_TYPES).find(t => t.name === type);
                return typeInfo && typeInfo.priority >= 0;
            })
            .sort((a, b) => {
                const priorityA = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === a)?.name] || {priority: 5};
                const priorityB = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === b)?.name] || {priority: 5};
                return priorityA.priority - priorityB.priority;
            });

        // 获取发布时间（使用 toTag 的时间）
        const releaseDate = this.getTagDate(toTag);

        let report = '';
        
        if (version) {
            report += `## ${version}\n\n`;
            report += `发布日期: ${releaseDate.toLocaleString('zh-CN')}\n\n`;
        } else {
            report += '## Unreleased\n\n';
        }

        sortedTypes.forEach(type => {
            const commits = groupedCommits[type];
            if (commits.length === 0) return;

            const emoji = commits[0].emoji;
            const label = EMOJI_TYPES[emoji]?.label || type;

            report += `### ${label}\n\n`;
            commits.forEach(commit => {
                report += `- ${commit.message} (${commit.hash}, @${commit.author})\n`;
            });
            report += '\n';
        });

        return report;
    }

    /**
     * 生成完整的 pnpm changeset 风格 changelog
     */
    generateCompleteChangelog() {
        const commitLines = this.getAllCommits();
        const parsedCommits = commitLines.map(line => this.parseCommit(line)).filter(Boolean);

        if (parsedCommits.length === 0) {
            return '## Unreleased\n\n没有找到符合规范的提交记录。\n';
        }

        const groupedCommits = this.groupCommitsByType(parsedCommits);

        // 按优先级排序的类型，过滤掉 priority < 0 的类型
        const sortedTypes = Object.keys(groupedCommits)
            .filter(type => {
                const typeInfo = Object.values(EMOJI_TYPES).find(t => t.name === type);
                return typeInfo && typeInfo.priority >= 0;
            })
            .sort((a, b) => {
                const priorityA = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === a)?.name] || {priority: 5};
                const priorityB = EMOJI_TYPES[Object.values(EMOJI_TYPES).find(t => t.name === b)?.name] || {priority: 5};
                return priorityA.priority - priorityB.priority;
            });

        let report = '## Unreleased\n\n';

        sortedTypes.forEach(type => {
            const commits = groupedCommits[type];
            if (commits.length === 0) return;

            const emoji = commits[0].emoji;
            const label = EMOJI_TYPES[emoji]?.label || type;

            report += `### ${label}\n\n`;
            commits.forEach(commit => {
                report += `- ${commit.message} (${commit.hash}, @${commit.author})\n`;
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
  --changelog     生成完整 changelog 模式
  --help, -h      显示帮助信息

示例:
  node generate-release-report.js v1.2.0
  node generate-release-report.js v1.2.0 --from v1.1.0
  node generate-release-report.js v1.2.0 --from v1.1.0 --output my-release.md
  node generate-release-report.js --changelog
`);
        return;
    }

    // 解析参数
    let version = null;
    let fromTag = null;
    let toTag = 'HEAD';
    let outputFile = null;
    let changelogMode = false;

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '--from') {
            fromTag = args[++i];
        } else if (arg === '--to') {
            toTag = args[++i];
        } else if (arg === '--output') {
            outputFile = args[++i];
        } else if (arg === '--changelog') {
            changelogMode = true;
        } else if (!arg.startsWith('--') && !version) {
            version = arg;
        }
    }

    const generator = new ReleaseReportGenerator();

    console.log(`🔄 正在生成 release 报告...`);

    try {
        let report;
        let outputPath;

        if (changelogMode) {
            // 生成完整的 changelog
            console.log(`📋 生成完整 changelog...`);
            report = generator.generateCompleteChangelog();
            
            if (!outputFile) {
                outputFile = 'CHANGELOG.md';
            }
            outputPath = generator.saveReport(report, outputFile);
        } else {
            // 生成单个 release 报告
            if (!version) {
                version = `v${new Date().toISOString().slice(0, 10)}`;
            }

            console.log(`📋 版本: ${version}`);
            if (fromTag) console.log(`📍 起始: ${fromTag}`);
            console.log(`📍 结束: ${toTag}`);
            console.log('');

            report = generator.generateMarkdownReport(version, fromTag, toTag);
            outputPath = generator.saveReport(report, outputFile);
        }

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
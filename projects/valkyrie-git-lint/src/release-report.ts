import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import { GitLintConfig, EmojiTypeConfig } from './config-loader.js';

export interface CommitInfo {
    hash: string;
    message: string;
    emoji?: string;
    type?: string;
    scope?: string;
    description: string;
    author?: string;
    date?: string;
}

export interface ParsedCommit {
    hash: string;
    message: string;
    emoji: string;
    type: string;
    scope?: string;
    subject: string;
    body?: string;
    author?: string;
    date?: string;
    description: string; // Added description to ParsedCommit
}

export interface GroupedCommits {
    [emoji: string]: ParsedCommit[];
}

export interface CommitStatistics {
    totalCommits: number;
    commitsByType: Record<string, number>;
    commitsByAuthor: Record<string, number>;
    dateRange: {
        from: string;
        to: string;
    };
}

export interface OutputConfig {
    directory?: string;
    dateFormat?: string;
    maxCommitsInPreview?: number;
    format?: string;
    filename?: string;
}

/**
 * 可配置的 Release Report 生成器
 * 基于原始 generate-release-report.js 功能，支持完全可配置
 */
export class ReleaseReportGenerator {
    private config: GitLintConfig;
    private emojiTypes: Record<string, EmojiTypeConfig>;

    constructor(config: GitLintConfig) {
        this.config = config;
        this.emojiTypes = config.emojiConfig;
    }

    /**
     * 获取输出配置
     */
    getOutputConfig(): OutputConfig {
        return this.config.releaseReport.output || {
            directory: 'releases',
            dateFormat: 'zh-CN',
            maxCommitsInPreview: 20,
            filename: 'CHANGELOG.md'
        };
    }

    /**
     * 获取两个 tag 之间的提交记录，包含作者信息
     */
    getCommitsBetweenTags(fromTag: string | null, toTag: string = 'HEAD'): string[] {
        try {
            // 先获取基本的提交信息
            const logCommand = fromTag
                ? `git log ${fromTag}..${toTag} --oneline`
                : `git log --oneline -${this.getOutputConfig().maxCommitsInPreview}`;
            
            const logOutput = execSync(logCommand, { encoding: 'utf8' });
            const commits = logOutput.trim().split('\n').filter(line => line.length > 0);

            // 为每个提交获取作者信息
            return commits.map(line => {
                const [hash] = line.split(' ');
                try {
                    const authorCommand = `git log -1 --format="%an|%ad" --date=short ${hash}`;
                    const authorOutput = execSync(authorCommand, { encoding: 'utf8' }).trim();
                    const [author, date] = authorOutput.split('|');
                    return `${line}|${author}|${date}`;
                } catch (error) {
                    return `${line}||`;
                }
            });
        } catch (error) {
            console.warn('获取提交记录失败:', error);
            return [];
        }
    }

    /**
     * 获取所有提交记录
     */
    getAllCommits(): string[] {
        try {
            const logOutput = execSync('git log --oneline', { encoding: 'utf8' });
            return logOutput.trim().split('\n').filter(line => line.length > 0).map(line => {
                const [hash] = line.split(' ');
                try {
                    const authorCommand = `git log -1 --format="%an|%ad" --date=short ${hash}`;
                    const authorOutput = execSync(authorCommand, { encoding: 'utf8' }).trim();
                    const [author, date] = authorOutput.split('|');
                    return `${line}|${author}|${date}`;
                } catch (error) {
                    return `${line}||`;
                }
            });
        } catch (error) {
            console.warn('获取所有提交记录失败:', error);
            return [];
        }
    }

    /**
     * 获取 tag 的日期
     */
    getTagDate(tag: string): string {
        try {
            return execSync(`git log -1 --format=%ai ${tag}`, { encoding: 'utf8' }).trim();
        } catch (error) {
            console.warn(`获取 tag ${tag} 日期失败:`, error);
            return '';
        }
    }

    /**
     * 解析提交信息
     * 支持多种格式：
     * 1. hash message|author|date (完整格式)
     * 2. hash message (简单格式)
     * 3. message (测试格式)
     */
    parseCommit(line: string): ParsedCommit | null {
        if (!line || line.trim() === '') {
            return null;
        }

        // 处理测试用例中的简单格式
        if (!line.includes('|') && !line.match(/^[a-f0-9]{7,}/)) {
            const emojiMatch = line.match(/^([\u{1F600}-\u{1F64F}]|[\u{1F300}-\u{1F5FF}]|[\u{1F680}-\u{1F6FF}]|[\u{1F1E0}-\u{1F1FF}]|[\u{2600}-\u{26FF}]|[\u{2700}-\u{27BF}])\s*(.+)/u);
            if (emojiMatch) {
                const emoji = emojiMatch[1];
                const subject = emojiMatch[2].trim();
                const typeConfig = this.emojiTypes[emoji];
                const type = typeConfig ? typeConfig.name : 'other';
                return {
                    hash: 'test',
                    message: line,
                    emoji: emoji,
                    type: type,
                    subject: subject,
                    description: subject
                };
            }
            return {
                hash: 'test',
                message: line,
                emoji: '',
                type: 'other',
                subject: line,
                description: line
            };
        }

        const parts = line.split('|');
        const commitPart = parts[0];
        const author = parts[1] || '';
        const date = parts[2] || '';

        // 提取 hash 和 message
        const hashMatch = commitPart.match(/^([a-f0-9]{7,})\s+(.+)/);
        if (!hashMatch) {
            return null;
        }

        const hash = hashMatch[1];
        const message = hashMatch[2];

        // 检查是否包含 emoji
        const emojiMatch = message.match(/^([\u{1F600}-\u{1F64F}]|[\u{1F300}-\u{1F5FF}]|[\u{1F680}-\u{1F6FF}]|[\u{1F1E0}-\u{1F1FF}]|[\u{2600}-\u{26FF}]|[\u{2700}-\u{27BF}])\s*(.+)/u);
        
        if (emojiMatch) {
            const emoji = emojiMatch[1];
            const subject = emojiMatch[2];
            
            // 查找对应的类型
            const typeConfig = this.emojiTypes[emoji];
            const type = typeConfig ? typeConfig.name : 'other';

            return {
                hash,
                message,
                emoji,
                type,
                subject,
                description: subject,
                author: author || undefined,
                date: date || undefined
            };
        }

        // 检查传统格式 type(scope): description
        const conventionalMatch = message.match(/^(\w+)(\(([^)]+)\))?: (.+)/);
        if (conventionalMatch) {
            return {
                hash,
                message,
                emoji: '',
                type: conventionalMatch[1],
                scope: conventionalMatch[3],
                subject: conventionalMatch[4],
                description: conventionalMatch[4],
                author: author || undefined,
                date: date || undefined
            };
        }

        // 默认情况
        return {
            hash,
            message,
            emoji: '',
            type: 'other',
            subject: message,
            description: message,
            author: author || undefined,
            date: date || undefined
        };
    }

    /**
     * 按类型分组提交
     */
    groupCommitsByType(commits: ParsedCommit[]): GroupedCommits {
        const grouped: GroupedCommits = {};

        for (const commit of commits) {
            let key = commit.type || 'other';
            
            if (!grouped[key]) {
                grouped[key] = [];
            }
            grouped[key].push(commit);
        }

        return grouped;
    }

    /**
     * 获取 emoji 类型配置
     */
    private getEmojiType(emoji: string): EmojiTypeConfig | undefined {
        return this.emojiTypes[emoji];
    }

    private getCommitTypeLabel(type: string): string {
        const emojiType = Object.values(this.emojiTypes).find(et => et.name === type);
        return emojiType ? emojiType.label || emojiType.name : type;
    }

    private sortCommitTypes(types: string[]): string[] {
        return types.sort((a, b) => {
            const emojiTypeA = Object.values(this.emojiTypes).find(et => et.name === a);
            const emojiTypeB = Object.values(this.emojiTypes).find(et => et.name === b);
            return (emojiTypeA?.order || 999) - (emojiTypeB?.order || 999);
        });
    }

    private formatCommit(commit: ParsedCommit): string {
        const emojiType = Object.values(this.emojiTypes).find(et => et.name === commit.type);
        const emoji = emojiType ? Object.keys(this.emojiTypes).find(key => this.emojiTypes[key] === emojiType) : '';
        const author = commit.author ? ` by @${commit.author}` : '';
        return `- ${emoji} **${commit.subject}** (${commit.hash.substring(0, 7)})${author}`;
    }

    /**
     * 生成 Markdown 格式的报告
     */
    generateMarkdown(version: string, groupedCommits: GroupedCommits): string {
        let markdown = `# Release ${version}\n\n`;

        const sortedTypes = this.sortCommitTypes(Object.keys(groupedCommits));

        for (const type of sortedTypes) {
            const commits = groupedCommits[type];
            if (commits.length > 0) {
                const typeLabel = this.getCommitTypeLabel(type);
                markdown += `## ${typeLabel}\n\n`;
                for (const commit of commits) {
                    markdown += `${this.formatCommit(commit)}\n`;
                }
                markdown += '\n';
            }
        }

        return markdown;
    }

    /**
     * 生成 release 报告
     */
    generateReleaseReport(version: string, fromTag: string | null = null, toTag: string = 'HEAD'): string {
        const commitLines = this.getCommitsBetweenTags(fromTag, toTag);
        const commits = commitLines.map(line => this.parseCommit(line)).filter((commit): commit is ParsedCommit => commit !== null);
        const groupedCommits = this.groupCommitsByType(commits);
        return this.generateMarkdown(version, groupedCommits);
    }

    /**
     * 生成完整的 changelog
     */
    generateChangelog(): string {
        try {
            // 获取所有 tags
            const tagsOutput = execSync('git tag --sort=-version:refname', { encoding: 'utf8' });
            const tags = tagsOutput.trim().split('\n').filter(tag => tag.length > 0);
            
            let changelog = '# Changelog\n\n';
            changelog += '本文档记录了项目的所有重要变更。\n\n';
            
            // 为每个版本生成 changelog
            for (let i = 0; i < tags.length; i++) {
                const currentTag = tags[i];
                const previousTag = tags[i + 1] || null;
                
                const tagDate = this.getTagDate(currentTag);
                const formattedDate = tagDate ? new Date(tagDate).toLocaleDateString('zh-CN') : '';
                
                changelog += `## [${currentTag}]${formattedDate ? ` - ${formattedDate}` : ''}\n\n`;
                
                const commitLines = this.getCommitsBetweenTags(previousTag, currentTag);
                const commits = commitLines.map(line => this.parseCommit(line)).filter((commit): commit is ParsedCommit => commit !== null);
                const groupedCommits = this.groupCommitsByType(commits);
                
                const sortedTypes = this.sortCommitTypes(Object.keys(groupedCommits));
                
                for (const type of sortedTypes) {
                    const typeCommits = groupedCommits[type];
                    if (typeCommits.length === 0) continue;
                    
                    const typeLabel = this.getCommitTypeLabel(type);
                    
                    changelog += `### ${typeLabel}\n\n`;
                    
                    for (const commit of typeCommits) {
                        changelog += `${this.formatCommit(commit)}\n`;
                    }
                    
                    changelog += '\n';
                }
                
                if (Object.keys(groupedCommits).length === 0) {
                    changelog += '无重要变更\n\n';
                }
            }
            
            return changelog;
        } catch (error) {
            console.error('生成完整 changelog 失败:', error);
            return '# Changelog\n\n生成失败，请检查 git 仓库状态。\n';
        }
    }

    /**
     * 保存报告到文件
     */
    saveReport(content: string, filename: string | null = null): string {
        const outputConfig = this.getOutputConfig();
        const outputDir = outputConfig.directory || 'releases';
        
        // 确保输出目录存在
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }
        
        const finalFilename = filename || outputConfig.filename || `release-${Date.now()}.md`;
        const filePath = path.join(outputDir, finalFilename);
        
        fs.writeFileSync(filePath, content, 'utf8');
        
        return filePath;
    }

    /**
     * 获取统计信息
     */
    getCommitStatistics(fromTag: string | null, toTag: string = 'HEAD'): CommitStatistics {
        const commitLines = this.getCommitsBetweenTags(fromTag, toTag);
        const commits = commitLines.map(line => this.parseCommit(line)).filter((commit): commit is ParsedCommit => commit !== null);
        
        const commitsByType: Record<string, number> = {};
        const commitsByAuthor: Record<string, number> = {};
        
        for (const commit of commits) {
            const type = commit.type || 'other';
            commitsByType[type] = (commitsByType[type] || 0) + 1;
            
            if (commit.author) {
                commitsByAuthor[commit.author] = (commitsByAuthor[commit.author] || 0) + 1;
            }
        }
        
        return {
            totalCommits: commits.length,
            commitsByType,
            commitsByAuthor,
            dateRange: {
                from: fromTag || 'beginning',
                to: toTag
            }
        };
    }
}

export default ReleaseReportGenerator;
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export interface CommitlintRule {
    [key: string]: [number, ...any[]];
}

export interface EmojiTypeConfig {
    name: string;
    label?: string;
    title: string;
    description: string;
    order: number;
}

export interface CommitlintConfig {
    extends?: string[];
    rules?: CommitlintRule;
    prompt?: {
        messages?: Record<string, string>;
        questions?: Record<string, any>;
    };
    emoji?: Record<string, EmojiTypeConfig>; // 新增 emoji 配置
}

export interface ReleaseReportConfig {
    enabled?: boolean;
    output?: {
        directory?: string;
        format?: string;
        filename?: string;
        dateFormat?: string;
        maxCommitsInPreview?: number;
    };
    template?: string; // 添加 template 属性
    emoji?: Record<string, EmojiTypeConfig>; // 新增 emoji 配置
    emptyCommits?: boolean; // 添加 emptyCommits 属性
}

export interface GitLintConfig {
    commitlint: CommitlintConfig;
    releaseReport: ReleaseReportConfig;
    emojiConfig: Record<string, EmojiTypeConfig>; // 顶层 emoji 配置
}

/**
 * 配置文件加载器
 * 负责加载和验证 git-lint.json 配置文件
 */
export class ConfigLoader {
    private defaultConfigPath: string;
    private configCache: Map<string, GitLintConfig>;

    constructor() {
        this.defaultConfigPath = path.join(__dirname, '..', 'git-lint.default.json');
        this.configCache = new Map();
    }

    /**
     * 查找配置文件
     * 按优先级查找：当前目录 -> 父目录递归查找 -> 默认配置
     */
    findConfigFile(startDir: string = process.cwd()): string {
        let currentDir = startDir;
        
        while (currentDir !== path.dirname(currentDir)) {
            const configPath = path.join(currentDir, 'git-lint.json');
            if (fs.existsSync(configPath)) {
                return configPath;
            }
            currentDir = path.dirname(currentDir);
        }
        
        // 如果没找到，返回默认配置路径
        return this.defaultConfigPath;
    }

    /**
     * 加载配置文件
     */
    loadConfig(configPath: string | null = null): GitLintConfig {
        const finalConfigPath = configPath || this.findConfigFile();
        
        // 检查缓存
        if (this.configCache.has(finalConfigPath)) {
            return this.configCache.get(finalConfigPath)!;
        }

        try {
            if (!fs.existsSync(finalConfigPath)) {
                throw new Error(`配置文件不存在: ${finalConfigPath}`);
            }

            const configContent = fs.readFileSync(finalConfigPath, 'utf8');
            const config: GitLintConfig = JSON.parse(configContent);
            
            // 验证配置格式
            this.validateConfig(config);
            
            // 缓存配置
            this.configCache.set(finalConfigPath, config);
            
            return config;
        } catch (error) {
            if (error instanceof SyntaxError) {
                throw new Error(`配置文件格式错误: ${error.message}`);
            }
            throw error;
        }
    }

    /**
     * 验证配置文件格式
     */
    validateConfig(config: any): void {
        if (!config || typeof config !== 'object') {
            throw new Error('配置文件必须是一个有效的 JSON 对象');
        }

        // 验证 commitlint 配置
        if (!config.commitlint) {
            throw new Error('缺少 commitlint 配置');
        }

        if (!config.commitlint.rules) {
            throw new Error('缺少 commitlint.rules 配置');
        }

        // 验证 releaseReport 配置
        if (!config.releaseReport) {
            throw new Error('缺少 releaseReport 配置');
        }

        // 验证顶层 emojiConfig 配置
        if (!config.emojiConfig) {
            throw new Error('缺少 emojiConfig 配置');
        }

        // 验证 emoji 类型配置
        for (const [emoji, typeConfig] of Object.entries(config.emojiConfig)) {
            const tc = typeConfig as any;
            if (!tc.name || !tc.label || typeof tc.priority !== 'number') {
                throw new Error(`emoji "${emoji}" 的配置格式错误，必须包含 name、label 和 priority 字段`);
            }
        }
    }

    /**
     * 获取 commitlint 配置
     */
    getCommitlintConfig(configPath: string | null = null): CommitlintConfig {
        const config = this.loadConfig(configPath);
        return config.commitlint;
    }

    /**
     * 获取 release report 配置
     */
    getReleaseReportConfig(configPath: string | null = null): ReleaseReportConfig {
        const config = this.loadConfig(configPath);
        return config.releaseReport;
    }

    /**
     * 清除缓存
     */
    clearCache(): void {
        this.configCache.clear();
    }

    /**
     * 创建默认配置文件
     */
    createDefaultConfig(configPath: string): void {
        const defaultConfig: GitLintConfig = {
            commitlint: {
                rules: {
                    'type-enum': [2, 'always', []],
                    'subject-empty': [2, 'never'],
                },
            },
            releaseReport: {
                output: {
                    directory: './release',
                    filename: 'release-report.md',
                    dateFormat: 'YYYY-MM-DD',
                },
                template: '',
                emptyCommits: false,
            },
            emojiConfig: {
                ':sparkles:': { name: 'feat', title: '✨ Features', description: '新功能', order: 1 },
                ':bug:': { name: 'fix', title: '🐛 Bug Fixes', description: 'Bug 修复', order: 2 },
                ':memo:': { name: 'docs', title: '📝 Documentation', description: '文档变更', order: 3 },
                ':lipstick:': { name: 'style', title: '💄 Styles', description: '代码格式 (不影响代码运行的变动)', order: 4 },
                ':recycle:': { name: 'refactor', title: '♻️ Code Refactoring', description: '代码重构 (不包括 bug 修复、功能新增)', order: 5 },
                ':zap:': { name: 'perf', title: '⚡ Performance Improvements', description: '性能优化', order: 6 },
                ':white_check_mark:': { name: 'test', title: '✅ Tests', description: '测试', order: 7 },
                ':wrench:': { name: 'build', title: '🔧 Build System', description: '构建过程或辅助工具的变动', order: 8 },
                ':construction_worker:': { name: 'ci', title: '👷 Continuous Integration', description: 'CI 配置、脚本的变更', order: 9 },
                ':rocket:': { name: 'chore', title: '🚀 Chores', description: '其他不修改 src 或 test 目录的提交', order: 10 },
                ':rewind:': { name: 'revert', title: '⏪ Reverts', description: '回滚 commit', order: 11 },
                ':package:': { name: 'dep', title: '📦 Dependency Updates', description: '依赖更新', order: 12 },
            },
        };

        // 确保 rules 和 type-enum 存在
        if (!defaultConfig.commitlint.rules) {
            defaultConfig.commitlint.rules = {};
        }
        if (!defaultConfig.commitlint.rules['type-enum']) {
            defaultConfig.commitlint.rules['type-enum'] = [2, 'always', []];
        }

        try {
            fs.writeFileSync(configPath, JSON.stringify(defaultConfig, null, 2), 'utf-8');
            console.log(`Default config created at ${configPath}`);
        } catch (error: any) {
            console.error(`Error creating default config: ${error.message}`);
        }
    }
}

export default ConfigLoader;
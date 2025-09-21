import { ConfigLoader, GitLintConfig } from './config-loader.js';
import ValkyrieGitLint, { LintResult } from './commit-lint.js';
import { ReleaseReportGenerator, CommitStatistics } from './release-report.js';

/**
 * Valkyrie Git Lint 主类
 * 封装了配置加载、commit 验证、release 报告生成等功能
 */
export class ValkyrieGitLintCli {
    private configPath: string;
    private configLoader: ConfigLoader;
    private config: GitLintConfig;
    private commitLint: ValkyrieGitLint;
    private releaseReportGenerator: ReleaseReportGenerator;

    constructor(configPath: string = './git-lint.json') {
        this.configPath = configPath;
        this.configLoader = new ConfigLoader();
        this.config = this.configLoader.loadConfig(configPath);
        this.commitLint = new ValkyrieGitLint(configPath);
        this.releaseReportGenerator = new ReleaseReportGenerator(this.config);
    }

    /**
     * 验证提交消息（支持传统格式和 emoji 格式）
     */
    validateCommitMessage(message: string): LintResult {
        return this.commitLint.validateGitCommit(message);
    }

    /**
     * 验证 emoji 格式的提交消息
     */
    validateEmojiCommitMessage(message: string): LintResult {
        return this.commitLint.validateGitCommit(message); // Assuming validateGitCommit handles both
    }

    /**
     * 生成 release 报告
     */
    generateReleaseReport(version: string, fromTag: string | null = null, toTag: string = 'HEAD'): string {
        return this.releaseReportGenerator.generateReleaseReport(version, fromTag, toTag);
    }

    /**
     * 生成完整的 changelog
     */
    generateChangelog(): string {
        const changelog = this.releaseReportGenerator.generateChangelog();
        console.log('Generated changelog content:');
        console.log(changelog);
        console.log('--- End of changelog ---');
        // 默认保存 changelog 文件
        const savedPath = this.releaseReportGenerator.saveReport(changelog);
        console.log('Saved changelog to:', savedPath);
        return changelog;
    }

    /**
     * 获取提交统计信息
     */
    getCommitStatistics(fromTag: string | null = null, toTag: string = 'HEAD'): CommitStatistics {
        return this.releaseReportGenerator.getCommitStatistics(fromTag, toTag);
    }

    /**
     * 获取允许的提交类型
     */
    getAllowedTypes(): string[] {
        return this.commitLint.getAllowedTypes();
    }

    /**
     * 获取当前配置
     */
    getConfig(): GitLintConfig {
        return this.config;
    }

    /**
     * 创建默认配置文件
     */
    createDefaultConfig(configPath: string): void {
        this.configLoader.createDefaultConfig(configPath);
    }
}

// 导出所有类
export { ConfigLoader } from './config-loader.js';
export { ValkyrieGitLint } from './commit-lint.js';
export { ReleaseReportGenerator } from './release-report.js';

// 默认导出
export default ValkyrieGitLintCli;
import { execSync } from 'child_process';
import { GitLintConfig, ConfigLoader, EmojiTypeConfig } from './config-loader';
import { ParsedCommit } from './release-report';

export interface LintResult {
    valid: boolean;
    errors: string[];
}

export class ValkyrieGitLint {
    private config: GitLintConfig;
    private emojiConfig: Record<string, EmojiTypeConfig>;

    constructor(configPath?: string) {
        const configLoader = new ConfigLoader();
        this.config = configLoader.loadConfig(configPath);
        this.emojiConfig = this.config.emojiConfig;
    }

    /**
     * 格式化提交信息，将 emoji 转换为对应的 type
     * @param message 原始提交信息
     * @returns 格式化后的提交信息
     */
    formatCommitMessage(message: string): string {
        for (const emoji in this.emojiConfig) {
            if (message.startsWith(emoji)) {
                return `${this.emojiConfig[emoji].name}${message.substring(emoji.length)}`;
            }
        }
        return message;
    }

    /**
     * 验证 Git 提交信息
     * @param commitMessage 提交信息
     * @returns 验证结果
     */
    validateGitCommit(commitMessage: string): LintResult {
        const formattedMessage = this.formatCommitMessage(commitMessage);
        const commitlintConfig = this.config.commitlint;

        // 模拟 commitlint 验证逻辑
        // 这里简化处理，实际 commitlint 有更复杂的规则解析和验证
        const errors: string[] = [];

        // 检查 type-enum 规则
        if (commitlintConfig.rules && commitlintConfig.rules['type-enum']) {
            const allowedTypes = commitlintConfig.rules['type-enum'][2] as string[];
            const commitType = formattedMessage.split(':')[0];
            if (!allowedTypes.includes(commitType)) {
                errors.push(`提交类型 '${commitType}' 不符合规范。允许的类型有: ${allowedTypes.join(', ')}`);
            }
        }

        // 检查 subject-empty 规则
        if (commitlintConfig.rules && commitlintConfig.rules['subject-empty']) {
            const subject = formattedMessage.split(':').slice(1).join(':').trim();
            const ruleValue = commitlintConfig.rules['subject-empty'][0];
            if (ruleValue === 2 && subject === '') {
                errors.push('提交主题不能为空。');
            }
        }

        return { valid: errors.length === 0, errors };
    }

    /**
     * 执行 lint 检查
     * @param commitHash 提交哈希 (可选，如果提供则只检查该提交)
     * @returns 检查结果
     */
    lint(commitHash?: string): LintResult {
        let commitMessages: string[] = [];

        if (commitHash) {
            // 如果提供了 commitHash，则只检查该提交
            try {
                const message = execSync(`git log -1 --pretty=%B ${commitHash}`, { encoding: 'utf8' }).trim();
                commitMessages.push(message);
            } catch (error: any) {
                return { valid: false, errors: [`无法获取提交信息: ${error.message}`] };
            }
        } else {
            // 否则检查所有未 lint 过的提交
            // 这里简化处理，实际应该获取所有需要检查的提交
            try {
                const log = execSync('git log --pretty=%B', { encoding: 'utf8' }).trim();
                commitMessages = log.split('\n\n').filter(msg => msg.trim() !== '');
            } catch (error: any) {
                return { valid: false, errors: [`无法获取 Git 日志: ${error.message}`] };
            }
        }

        const allErrors: string[] = [];
        let allValid = true;

        for (const message of commitMessages) {
            const { valid, errors } = this.validateGitCommit(message);
            if (!valid) {
                allValid = false;
                allErrors.push(`提交信息 '${message}' 验证失败:`);
                allErrors.push(...errors.map(e => `  - ${e}`));
            }
        }

        return { valid: allValid, errors: allErrors };
    }

    /**
     * 获取允许的提交类型
     */
    getAllowedTypes(): string[] {
        const commitlintConfig = this.config.commitlint;
        if (commitlintConfig.rules && commitlintConfig.rules['type-enum']) {
            return commitlintConfig.rules['type-enum'][2] as string[];
        }
        return [];
    }

    /**
     * 运行 changelog 命令
     * @returns changelog 内容
     */
    runChangelogCommand(): string {
        // 这里简化处理，实际应该根据配置生成 changelog
        return 'Changelog generation not yet implemented.';
    }
}

export default ValkyrieGitLint;
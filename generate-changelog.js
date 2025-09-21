// 包装脚本，确保 changelog 输出能正确显示
import { fileURLToPath, pathToFileURL } from 'url';
import { dirname, join } from 'path';
import { writeFileSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 导入 git-lint 模块
const gitLintPath = pathToFileURL(join(__dirname, 'projects/valkyrie-git-lint/dist/git-lint.js')).href;

try {
  const { default: CLI } = await import(gitLintPath);
  
  console.log('生成 changelog...');
  const cli = new CLI();
  const changelog = cli.gitLint.generateChangelog();
  
  // 直接输出到控制台
  console.log(changelog);
  
  // 同时保存到文件作为备份
  const outputPath = join(__dirname, 'CHANGELOG.md');
  writeFileSync(outputPath, changelog, 'utf8');
  console.log(`\nChangelog 已保存到: ${outputPath}`);
  
} catch (error) {
  console.error('生成 changelog 失败:', error);
  process.exit(1);
}
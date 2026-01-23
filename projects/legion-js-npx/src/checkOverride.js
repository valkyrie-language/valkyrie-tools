import fs from "fs-extra";
import prompts from "prompts";
import chalk from "chalk";

/**
 * @param {string} target
 * @returns {Promise<boolean>}
 * */
export async function checkOverride (target) {
    let isOverride = false;
    if (fs.existsSync(target)) {
        const result = await prompts({
            name: "yes",
            type: "toggle",
            message: chalk.bold(`Do you want to remove old `, chalk.underline(`'${target}'`), '?'),
            initial: true,
            active: 'yes',
            inactive: 'no!'
        });
        isOverride = result.yes;
        if (!isOverride) process.exit(1);
        
        console.log(`Removing ${chalk.cyan(target)}...`);
        await fs.remove(target);
    }
    fs.mkdirpSync(target, null);
    return isOverride
}
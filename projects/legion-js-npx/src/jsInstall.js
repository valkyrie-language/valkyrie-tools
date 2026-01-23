import prompts from "prompts";
import {execSync} from "child_process";

/**
 * @typedef JsInstall
 * @property {string} manager The node package manager
 */

/**
 * @param {string} target Target directory
 * @returns {Promise<JsInstall>}
 */
export async function jsInstall (target) {
    const cmdOptions = {
        stdio: "inherit",
        cwd: target,
    }
    const {manager} = await prompts([
        {
            name: "manager",
            type: "select",
            initial: 1,
            message: "Select the package manager",
            choices: [
                {
                    title: "💙 pnpm",
                    value: 'pnpm',
                },
                {
                    title: "💚 npm",
                    value: 'npm',
                },
                {
                    title: "💛 yarn",
                    value: 'yarn',
                },
            ],
        }
    ]);
    const options = {
        manager
    }
    if (manager === "npm") {
        execSync("npm i", cmdOptions);
    } else if (manager === "pnpm") {
        execSync("pnpm i", cmdOptions);
    } else if (manager === "yarn") {
        execSync("yarn", cmdOptions);
    }
    return options
}
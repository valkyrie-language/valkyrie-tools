import fs from "fs-extra";

import prompts from "prompts";

import {validateName} from "./src/validateName.js";
import chalk from "chalk";
import {copyFile, cp, readFile, writeFile} from "node:fs/promises";
import {basename, dirname, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import * as constants from "node:constants";
import {execSync} from "child_process";
import {jsInstall} from "./src/jsInstall.js";
import {checkOverride} from "./src/checkOverride.js";
import {copyTemplate} from "./src/copyTemplate.js";

/**
 * @param {string} target Target directory
 * @param {string} name Package name
 * @returns {Promise<void>}
 */
export async function makeTemplate (target, name) {
    validateName(name);
    await checkOverride(target);
    const here = dirname(fileURLToPath(import.meta.url))
    
    await copyTemplate(target, here, name);
    await jsInstall(target);
    
    console.log();
    console.log(
        `${chalk.green("✔")} Success! Created ${chalk.cyan(
            name
        )} at ${chalk.cyan(target)}`
    );
    console.log();
}


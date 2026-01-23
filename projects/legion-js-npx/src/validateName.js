import {execSync} from "child_process";

import chalk from "chalk";

import validateProjectName from "validate-npm-package-name";

/**
 * @param {string} name Input name
 * @returns {void}
 */
export function validateName (name) {
    const validationResult = validateProjectName(name);
    
    function printValidationResults (results) {
        if (typeof results !== "undefined") {
            results.forEach((error) => {
                console.error(chalk.red(`  *  ${error}`));
            });
        }
    }
    
    if (!validationResult.validForNewPackages) {
        console.error(
            `Could not create a project called ${chalk.red(
                `"${name}"`
            )} because of npm naming restrictions:`
        );
        printValidationResults(validationResult.errors);
        printValidationResults(validationResult.warnings);
        
        process.exit(1);
    }
}

import prompts from "prompts";
import {basename, dirname, resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {cp, readFile, writeFile} from "node:fs/promises";
import chalk from "chalk";

/**
 * @typedef CopyTemplate
 * @property {string} name Project name
 * @property {boolean} useWorkspace Use workspace or not
 * @property {boolean} isBinary Is binary or not
 */


/**
 * @param {string} target Target directory
 * @param {string} here Current directory
 * @param {string} pkg Package name
 * @returns {Promise<CopyTemplate>}
 */
export async function copyTemplate (target, here, pkg) {
    const {name, useWorkspace} = await prompts([
        {
            name: "name",
            type: "text",
            message: "What is the project name?",
            initial: pkg,
        },
        {
            name: "useWorkspace",
            type: "select",
            initial: 1,
            message: "Do you want to make a workspace?",
            choices: [
                {
                    title: "📚 Workspace",
                    value: true,
                },
                {
                    title: "📗 Package",
                    value: false,
                },
            ],
        }
    ]);
    let isBinary = false;
    
    if (!useWorkspace) {
        const {isLibrary} = await prompts([
                {
                    name: "isLibrary",
                    type: "select",
                    initial: 0,
                    message: "What kind of project is this?",
                    choices: [
                        {
                            title: "🎓 Library",
                            value: false,
                        },
                        {
                            title: "📟 Commands",
                            value: true,
                        },
                    ],
                }
            ]
        )
        isBinary = !isLibrary
    }
    const options = {
        name,
        useWorkspace,
        isBinary
    }
    
    let source;
    if (options.useWorkspace) {
        source = resolve(here, "template/workspace");
    } else {
        source = resolve(here, "template/single");
    }
    console.log(`Coping `, chalk.greenBright(source))
    await cp(source, target, {recursive: true});
    
    // add readme
    await writeFile(resolve(target, "readme.md"), `# ${options.name}\n\n${112233}\n`);
    const legion = resolve(target, "legion.json");
    // add info to json
    const json = JSON.parse(await readFile(legion, {encoding: 'utf-8'}));
    //
    const bName = basename(target);
    json.name = options.name;
    // json.description = desc;
    json.main = `dist/${bName}.cjs.js`;
    json.module = `dist/${bName}.esm.js`;
    await writeFile(legion, JSON.stringify(json, null, 4) + "\n");
    return options
}
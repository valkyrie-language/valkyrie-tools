#!/usr/bin/env node

import {makeTemplate} from "./index.js";
import {resolve} from "node:path";

const pkgName = process.argv[2];
let target;
if (pkgName.startsWith("@")) {
    target = resolve(
        pkgName.split("/")[1]
    );
} else {
    target = resolve(pkgName);
}
await makeTemplate(target, pkgName)
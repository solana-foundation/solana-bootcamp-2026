"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const toml = __importStar(require("toml"));
const camelcase_1 = __importDefault(require("camelcase"));
const index_js_1 = require("./program/index.js");
const common_js_1 = require("./utils/common.js");
/**
 * The `workspace` namespace provides a convenience API to automatically
 * search for and deserialize [[Program]] objects defined by compiled IDLs
 * in an Anchor workspace.
 *
 * This API is for Node only.
 */
const workspace = new Proxy({}, {
    get(workspaceCache, programName) {
        var _a, _b;
        if (common_js_1.isBrowser) {
            throw new Error("Workspaces aren't available in the browser");
        }
        // Converting `programName` to camelCase enables the ability to use any
        // of the following to access the workspace program:
        // `workspace.myProgram`, `workspace.MyProgram`, `workspace["my-program"]`...
        programName = (0, camelcase_1.default)(programName);
        // Return early if the program is in cache
        if (workspaceCache[programName])
            return workspaceCache[programName];
        const fs = require("fs");
        const path = require("path");
        // Override the workspace programs if the user put them in the config.
        const anchorToml = toml.parse(fs.readFileSync("Anchor.toml"));
        const clusterId = anchorToml.provider.cluster;
        const programs = (_a = anchorToml.programs) === null || _a === void 0 ? void 0 : _a[clusterId];
        let programEntry;
        if (programs) {
            programEntry = (_b = Object.entries(programs).find(([key]) => (0, camelcase_1.default)(key) === programName)) === null || _b === void 0 ? void 0 : _b[1];
        }
        let idlPath;
        let programId;
        if (typeof programEntry === "object" && programEntry.idl) {
            idlPath = programEntry.idl;
            programId = programEntry.address;
        }
        else {
            // Assuming the IDL file's name to be the snake_case name of the
            // `programName` with `.json` extension results in problems when
            // numbers are involved due to the nature of case conversion from
            // camelCase to snake_case being lossy.
            //
            // To avoid the above problem with numbers, read the `idl` directory and
            // compare the camelCased  version of both file names and `programName`.
            const idlDirPath = path.join("target", "idl");
            const fileName = fs
                .readdirSync(idlDirPath)
                .find((name) => (0, camelcase_1.default)(path.parse(name).name) === programName);
            if (!fileName) {
                throw new Error(`Failed to find IDL of program \`${programName}\``);
            }
            idlPath = path.join(idlDirPath, fileName);
        }
        if (!fs.existsSync(idlPath)) {
            throw new Error(`${idlPath} doesn't exist. Did you run \`anchor build\`?`);
        }
        const idl = JSON.parse(fs.readFileSync(idlPath));
        if (programId) {
            idl.address = programId;
        }
        workspaceCache[programName] = new index_js_1.Program(idl);
        return workspaceCache[programName];
    },
});
exports.default = workspace;
//# sourceMappingURL=workspace.js.map
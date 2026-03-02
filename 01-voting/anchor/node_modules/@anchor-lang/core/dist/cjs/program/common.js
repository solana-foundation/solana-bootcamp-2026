"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseIdlErrors = parseIdlErrors;
exports.toInstruction = toInstruction;
exports.validateAccounts = validateAccounts;
exports.translateAddress = translateAddress;
const web3_js_1 = require("@solana/web3.js");
const idl_js_1 = require("../idl.js");
function parseIdlErrors(idl) {
    const errors = new Map();
    if (idl.errors) {
        idl.errors.forEach((e) => {
            var _a;
            let msg = (_a = e.msg) !== null && _a !== void 0 ? _a : e.name;
            errors.set(e.code, msg);
        });
    }
    return errors;
}
function toInstruction(idlIx, ...args) {
    if (idlIx.args.length != args.length) {
        throw new Error("Invalid argument length");
    }
    const ix = {};
    let idx = 0;
    idlIx.args.forEach((ixArg) => {
        ix[ixArg.name] = args[idx];
        idx += 1;
    });
    return ix;
}
// Throws error if any account required for the `ix` is not given.
function validateAccounts(ixAccounts, accounts = {}) {
    ixAccounts.forEach((acc) => {
        if ((0, idl_js_1.isCompositeAccounts)(acc)) {
            validateAccounts(acc.accounts, accounts[acc.name]);
        }
        else {
            if (!accounts[acc.name]) {
                throw new Error(`Account \`${acc.name}\` not provided.`);
            }
        }
    });
}
// Translates an address to a Pubkey.
function translateAddress(address) {
    return address instanceof web3_js_1.PublicKey ? address : new web3_js_1.PublicKey(address);
}
//# sourceMappingURL=common.js.map
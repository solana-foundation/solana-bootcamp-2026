"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.BorshAccountsCoder = void 0;
const bs58_1 = __importDefault(require("bs58"));
const buffer_1 = require("buffer");
const idl_js_1 = require("./idl.js");
/**
 * Encodes and decodes account objects.
 */
class BorshAccountsCoder {
    constructor(idl) {
        this.idl = idl;
        if (!idl.accounts) {
            this.accountLayouts = new Map();
            return;
        }
        const types = idl.types;
        if (!types) {
            throw new Error("Accounts require `idl.types`");
        }
        const layouts = idl.accounts.map((acc) => {
            const typeDef = types.find((ty) => ty.name === acc.name);
            if (!typeDef) {
                throw new Error(`Account not found: ${acc.name}`);
            }
            return [
                acc.name,
                {
                    discriminator: acc.discriminator,
                    layout: idl_js_1.IdlCoder.typeDefLayout({ typeDef, types }),
                },
            ];
        });
        this.accountLayouts = new Map(layouts);
    }
    async encode(accountName, account) {
        const buffer = buffer_1.Buffer.alloc(1000); // TODO: use a tighter buffer.
        const layout = this.accountLayouts.get(accountName);
        if (!layout) {
            throw new Error(`Unknown account: ${accountName}`);
        }
        const len = layout.layout.encode(account, buffer);
        const accountData = buffer.slice(0, len);
        const discriminator = this.accountDiscriminator(accountName);
        return buffer_1.Buffer.concat([discriminator, accountData]);
    }
    decode(accountName, data) {
        // Assert the account discriminator is correct.
        const discriminator = this.accountDiscriminator(accountName);
        if (discriminator.compare(data.slice(0, discriminator.length))) {
            throw new Error("Invalid account discriminator");
        }
        return this.decodeUnchecked(accountName, data);
    }
    decodeAny(data) {
        for (const [name, layout] of this.accountLayouts) {
            const givenDisc = data.subarray(0, layout.discriminator.length);
            const matches = givenDisc.equals(buffer_1.Buffer.from(layout.discriminator));
            if (matches)
                return this.decodeUnchecked(name, data);
        }
        throw new Error("Account not found");
    }
    decodeUnchecked(accountName, acc) {
        // Chop off the discriminator before decoding.
        const discriminator = this.accountDiscriminator(accountName);
        const data = acc.subarray(discriminator.length);
        const layout = this.accountLayouts.get(accountName);
        if (!layout) {
            throw new Error(`Unknown account: ${accountName}`);
        }
        return layout.layout.decode(data);
    }
    memcmp(accountName, appendData) {
        const discriminator = this.accountDiscriminator(accountName);
        return {
            offset: 0,
            bytes: bs58_1.default.encode(appendData ? buffer_1.Buffer.concat([discriminator, appendData]) : discriminator),
        };
    }
    size(accountName) {
        return (this.accountDiscriminator(accountName).length +
            idl_js_1.IdlCoder.typeSize({ defined: { name: accountName } }, this.idl));
    }
    /**
     * Get the unique discriminator prepended to all anchor accounts.
     *
     * @param name The name of the account to get the discriminator of.
     */
    accountDiscriminator(name) {
        var _a;
        const account = (_a = this.idl.accounts) === null || _a === void 0 ? void 0 : _a.find((acc) => acc.name === name);
        if (!account) {
            throw new Error(`Account not found: ${name}`);
        }
        return buffer_1.Buffer.from(account.discriminator);
    }
}
exports.BorshAccountsCoder = BorshAccountsCoder;
//# sourceMappingURL=accounts.js.map
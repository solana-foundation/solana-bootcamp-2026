"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MethodsBuilder = exports.MethodsBuilderFactory = void 0;
exports.isPartialAccounts = isPartialAccounts;
exports.flattenPartialAccounts = flattenPartialAccounts;
const accounts_resolver_js_1 = require("../accounts-resolver.js");
const common_js_1 = require("../common.js");
class MethodsBuilderFactory {
    static build(provider, programId, idlIx, ixFn, txFn, rpcFn, simulateFn, viewFn, accountNamespace, idlTypes, customResolver) {
        return (...args) => new MethodsBuilder(args, ixFn, txFn, rpcFn, simulateFn, viewFn, provider, programId, idlIx, accountNamespace, idlTypes, customResolver);
    }
}
exports.MethodsBuilderFactory = MethodsBuilderFactory;
function isPartialAccounts(partialAccount) {
    return (typeof partialAccount === "object" &&
        partialAccount !== null &&
        !("_bn" in partialAccount) // Ensures not a pubkey
    );
}
function flattenPartialAccounts(partialAccounts, throwOnNull) {
    const toReturn = {};
    for (const accountName in partialAccounts) {
        const account = partialAccounts[accountName];
        if (account === null) {
            if (throwOnNull)
                throw new Error("Failed to resolve optionals due to IDL type mismatch with input accounts!");
            continue;
        }
        toReturn[accountName] = isPartialAccounts(account)
            ? flattenPartialAccounts(account, true)
            : (0, common_js_1.translateAddress)(account);
    }
    return toReturn;
}
class MethodsBuilder {
    constructor(_args, _ixFn, _txFn, _rpcFn, _simulateFn, _viewFn, provider, programId, idlIx, accountNamespace, idlTypes, customResolver) {
        this._args = _args;
        this._ixFn = _ixFn;
        this._txFn = _txFn;
        this._rpcFn = _rpcFn;
        this._simulateFn = _simulateFn;
        this._viewFn = _viewFn;
        this._accounts = {};
        this._remainingAccounts = [];
        this._signers = [];
        this._preInstructions = [];
        this._postInstructions = [];
        this._resolveAccounts = true;
        this._accountsResolver = new accounts_resolver_js_1.AccountsResolver(_args, this._accounts, provider, programId, idlIx, accountNamespace, idlTypes, customResolver);
    }
    args(args) {
        this._args = args;
        this._accountsResolver.args(args);
    }
    /**
     * Set instruction accounts with account resolution.
     *
     * This method only accepts accounts that cannot be resolved.
     *
     * See {@link accountsPartial} for overriding the account resolution or
     * {@link accountsStrict} for strictly specifying all accounts.
     */
    accounts(accounts) {
        // @ts-ignore
        return this.accountsPartial(accounts);
    }
    /**
     * Set instruction accounts with account resolution.
     *
     * There is no functional difference between this method and {@link accounts}
     * method, the only difference is this method allows specifying all accounts
     * even if they can be resolved. On the other hand, {@link accounts} method
     * doesn't accept accounts that can be resolved.
     */
    accountsPartial(accounts) {
        this._resolveAccounts = true;
        this._accountsResolver.resolveOptionals(accounts);
        return this;
    }
    /**
     * Set instruction accounts without account resolution.
     *
     * All accounts strictly need to be specified when this method is used.
     *
     * See {@link accounts} and {@link accountsPartial} methods for automatically
     * resolving accounts.
     *
     * @param accounts instruction accounts
     */
    accountsStrict(accounts) {
        this._resolveAccounts = false;
        this._accountsResolver.resolveOptionals(accounts);
        return this;
    }
    /**
     * Set instruction signers.
     *
     * Note that calling this method appends the given signers to the existing
     * signers (instead of overriding them).
     *
     * @param signers signers to append
     */
    signers(signers) {
        this._signers = this._signers.concat(signers);
        return this;
    }
    /**
     * Set remaining accounts.
     *
     * Note that calling this method appends the given accounts to the existing
     * remaining accounts (instead of overriding them).
     *
     * @param accounts remaining accounts
     */
    remainingAccounts(accounts) {
        this._remainingAccounts = this._remainingAccounts.concat(accounts);
        return this;
    }
    /**
     * Set previous instructions.
     *
     * See {@link postInstructions} to set the post instructions instead.
     *
     * @param ixs instructions
     * @param prepend whether to prepend to the existing previous instructions
     */
    preInstructions(ixs, prepend = false) {
        if (prepend) {
            this._preInstructions = ixs.concat(this._preInstructions);
        }
        else {
            this._preInstructions = this._preInstructions.concat(ixs);
        }
        return this;
    }
    /**
     * Set post instructions.
     *
     * See {@link preInstructions} to set the previous instructions instead.
     *
     * @param ixs instructions
     */
    postInstructions(ixs) {
        this._postInstructions = this._postInstructions.concat(ixs);
        return this;
    }
    /**
     * Get the public keys of the instruction accounts.
     *
     * The return type is an object with account names as keys and their public
     * keys as their values.
     *
     * Note that an account key is `undefined` if the account hasn't yet been
     * specified or resolved.
     */
    async pubkeys() {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        // @ts-ignore
        return this._accounts;
    }
    /**
     * Create an instruction based on the current configuration.
     *
     * See {@link transaction} to create a transaction instead.
     *
     * @returns the transaction instruction
     */
    async instruction() {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        // @ts-ignore
        return this._ixFn(...this._args, {
            accounts: this._accounts,
            signers: this._signers,
            remainingAccounts: this._remainingAccounts,
            preInstructions: this._preInstructions,
            postInstructions: this._postInstructions,
        });
    }
    /**
     * Create a transaction based on the current configuration.
     *
     * This method doesn't send the created transaction. Use {@link rpc} method
     * to conveniently send an confirm the configured transaction.
     *
     * See {@link instruction} to only create an instruction instead.
     *
     * @returns the transaction
     */
    async transaction() {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        // @ts-ignore
        return this._txFn(...this._args, {
            accounts: this._accounts,
            signers: this._signers,
            remainingAccounts: this._remainingAccounts,
            preInstructions: this._preInstructions,
            postInstructions: this._postInstructions,
        });
    }
    /**
     * Simulate the configured transaction.
     *
     * @param options confirmation options
     * @returns the simulation response
     */
    async simulate(options) {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        // @ts-ignore
        return this._simulateFn(...this._args, {
            accounts: this._accounts,
            signers: this._signers,
            remainingAccounts: this._remainingAccounts,
            preInstructions: this._preInstructions,
            postInstructions: this._postInstructions,
            options,
        });
    }
    /**
     * View the configured transaction.
     *
     * Note that to use this method, the instruction needs to return a value and
     * all its accounts must be read-only.
     *
     * @param options confirmation options
     * @returns the return value of the instruction
     */
    async view(options) {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        if (!this._viewFn) {
            throw new Error([
                "Method does not support views.",
                "The instruction should return a value, and its accounts must be read-only",
            ].join(" "));
        }
        // @ts-ignore
        return this._viewFn(...this._args, {
            accounts: this._accounts,
            signers: this._signers,
            remainingAccounts: this._remainingAccounts,
            preInstructions: this._preInstructions,
            postInstructions: this._postInstructions,
            options,
        });
    }
    /**
     * Send and confirm the configured transaction.
     *
     * See {@link rpcAndKeys} to both send the transaction and get the resolved
     * account public keys.
     *
     * @param options confirmation options
     * @returns the transaction signature
     */
    async rpc(options) {
        if (this._resolveAccounts) {
            await this._accountsResolver.resolve();
        }
        // @ts-ignore
        return this._rpcFn(...this._args, {
            accounts: this._accounts,
            signers: this._signers,
            remainingAccounts: this._remainingAccounts,
            preInstructions: this._preInstructions,
            postInstructions: this._postInstructions,
            options,
        });
    }
    /**
     * Conveniently call both {@link rpc} and {@link pubkeys} methods.
     *
     * @param options confirmation options
     * @returns the transaction signature and account public keys
     */
    async rpcAndKeys(options) {
        return {
            signature: await this.rpc(options),
            pubkeys: (await this.pubkeys()),
        };
    }
    /**
     * Get instruction information necessary to include the instruction inside a
     * transaction.
     *
     * # Example
     *
     * ```ts
     * const { instruction, signers, pubkeys } = await method.prepare();
     * ```
     */
    async prepare() {
        return {
            instruction: await this.instruction(),
            signers: this._signers,
            pubkeys: await this.pubkeys(),
        };
    }
}
exports.MethodsBuilder = MethodsBuilder;
//# sourceMappingURL=methods.js.map
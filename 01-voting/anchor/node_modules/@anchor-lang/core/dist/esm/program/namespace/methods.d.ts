import { AccountMeta, ConfirmOptions, PublicKey, Signer, Transaction, TransactionInstruction, TransactionSignature } from "@solana/web3.js";
import { Idl, IdlInstructionAccount, IdlInstructionAccountItem, IdlInstructionAccounts, IdlTypeDef } from "../../idl.js";
import Provider from "../../provider.js";
import { AccountsGeneric, CustomAccountResolver } from "../accounts-resolver.js";
import { Address } from "../common.js";
import { Accounts } from "../context.js";
import { AccountNamespace } from "./account.js";
import { InstructionFn } from "./instruction.js";
import { RpcFn } from "./rpc.js";
import { SimulateFn, SimulateResponse } from "./simulate.js";
import { TransactionFn } from "./transaction.js";
import { AllInstructions, InstructionAccountAddresses, MakeMethodsNamespace, MethodsFn } from "./types.js";
import { ViewFn } from "./views.js";
export type MethodsNamespace<IDL extends Idl = Idl, I extends AllInstructions<IDL> = AllInstructions<IDL>> = MakeMethodsNamespace<IDL, I>;
export declare class MethodsBuilderFactory {
    static build<IDL extends Idl, I extends AllInstructions<IDL>>(provider: Provider, programId: PublicKey, idlIx: AllInstructions<IDL>, ixFn: InstructionFn<IDL>, txFn: TransactionFn<IDL>, rpcFn: RpcFn<IDL>, simulateFn: SimulateFn<IDL>, viewFn: ViewFn<IDL> | undefined, accountNamespace: AccountNamespace<IDL>, idlTypes: IdlTypeDef[], customResolver?: CustomAccountResolver<IDL>): MethodsFn<IDL, I, MethodsBuilder<IDL, I>>;
}
type ResolvedAccounts<A extends IdlInstructionAccountItem = IdlInstructionAccountItem> = PartialUndefined<ResolvedAccountsRecursive<A>>;
type ResolvedAccountsRecursive<A extends IdlInstructionAccountItem = IdlInstructionAccountItem> = OmitNever<{
    [N in A["name"]]: ResolvedAccount<A & {
        name: N;
    }>;
}>;
type ResolvedAccount<A extends IdlInstructionAccountItem = IdlInstructionAccountItem> = A extends IdlInstructionAccounts ? ResolvedAccountsRecursive<A["accounts"][number]> : A extends NonNullable<Pick<IdlInstructionAccount, "address">> ? never : A extends NonNullable<Pick<IdlInstructionAccount, "pda">> ? never : A extends NonNullable<Pick<IdlInstructionAccount, "relations">> ? never : A extends {
    signer: true;
} ? Address | undefined : PartialAccount<A>;
type PartialUndefined<T, P extends keyof T = {
    [K in keyof T]: undefined extends T[K] ? K : never;
}[keyof T]> = Partial<Pick<T, P>> & Pick<T, Exclude<keyof T, P>>;
type OmitNever<T extends Record<string, any>> = {
    [K in keyof T as T[K] extends never ? never : K]: T[K];
};
export type PartialAccounts<A extends IdlInstructionAccountItem = IdlInstructionAccountItem> = Partial<{
    [N in A["name"]]: PartialAccount<A & {
        name: N;
    }>;
}>;
type PartialAccount<A extends IdlInstructionAccountItem = IdlInstructionAccountItem> = A extends IdlInstructionAccounts ? PartialAccounts<A["accounts"][number]> : A extends {
    optional: true;
} ? Address | null : Address;
export declare function isPartialAccounts(partialAccount: any): partialAccount is PartialAccounts;
export declare function flattenPartialAccounts<A extends IdlInstructionAccountItem>(partialAccounts: PartialAccounts<A>, throwOnNull: boolean): AccountsGeneric;
export declare class MethodsBuilder<IDL extends Idl, I extends AllInstructions<IDL>, A extends I["accounts"][number] = I["accounts"][number]> {
    private _args;
    private _ixFn;
    private _txFn;
    private _rpcFn;
    private _simulateFn;
    private _viewFn;
    private _accounts;
    private _remainingAccounts;
    private _signers;
    private _preInstructions;
    private _postInstructions;
    private _accountsResolver;
    private _resolveAccounts;
    constructor(_args: Array<any>, _ixFn: InstructionFn<IDL>, _txFn: TransactionFn<IDL>, _rpcFn: RpcFn<IDL>, _simulateFn: SimulateFn<IDL>, _viewFn: ViewFn<IDL> | undefined, provider: Provider, programId: PublicKey, idlIx: AllInstructions<IDL>, accountNamespace: AccountNamespace<IDL>, idlTypes: IdlTypeDef[], customResolver?: CustomAccountResolver<IDL>);
    args(args: Array<any>): void;
    /**
     * Set instruction accounts with account resolution.
     *
     * This method only accepts accounts that cannot be resolved.
     *
     * See {@link accountsPartial} for overriding the account resolution or
     * {@link accountsStrict} for strictly specifying all accounts.
     */
    accounts(accounts: ResolvedAccounts<A>): this;
    /**
     * Set instruction accounts with account resolution.
     *
     * There is no functional difference between this method and {@link accounts}
     * method, the only difference is this method allows specifying all accounts
     * even if they can be resolved. On the other hand, {@link accounts} method
     * doesn't accept accounts that can be resolved.
     */
    accountsPartial(accounts: PartialAccounts<A>): this;
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
    accountsStrict(accounts: Accounts<A>): this;
    /**
     * Set instruction signers.
     *
     * Note that calling this method appends the given signers to the existing
     * signers (instead of overriding them).
     *
     * @param signers signers to append
     */
    signers(signers: Array<Signer>): this;
    /**
     * Set remaining accounts.
     *
     * Note that calling this method appends the given accounts to the existing
     * remaining accounts (instead of overriding them).
     *
     * @param accounts remaining accounts
     */
    remainingAccounts(accounts: Array<AccountMeta>): this;
    /**
     * Set previous instructions.
     *
     * See {@link postInstructions} to set the post instructions instead.
     *
     * @param ixs instructions
     * @param prepend whether to prepend to the existing previous instructions
     */
    preInstructions(ixs: Array<TransactionInstruction>, prepend?: boolean): this;
    /**
     * Set post instructions.
     *
     * See {@link preInstructions} to set the previous instructions instead.
     *
     * @param ixs instructions
     */
    postInstructions(ixs: Array<TransactionInstruction>): this;
    /**
     * Get the public keys of the instruction accounts.
     *
     * The return type is an object with account names as keys and their public
     * keys as their values.
     *
     * Note that an account key is `undefined` if the account hasn't yet been
     * specified or resolved.
     */
    pubkeys(): Promise<Partial<InstructionAccountAddresses<IDL, I>>>;
    /**
     * Create an instruction based on the current configuration.
     *
     * See {@link transaction} to create a transaction instead.
     *
     * @returns the transaction instruction
     */
    instruction(): Promise<TransactionInstruction>;
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
    transaction(): Promise<Transaction>;
    /**
     * Simulate the configured transaction.
     *
     * @param options confirmation options
     * @returns the simulation response
     */
    simulate(options?: ConfirmOptions): Promise<SimulateResponse>;
    /**
     * View the configured transaction.
     *
     * Note that to use this method, the instruction needs to return a value and
     * all its accounts must be read-only.
     *
     * @param options confirmation options
     * @returns the return value of the instruction
     */
    view(options?: ConfirmOptions): Promise<any>;
    /**
     * Send and confirm the configured transaction.
     *
     * See {@link rpcAndKeys} to both send the transaction and get the resolved
     * account public keys.
     *
     * @param options confirmation options
     * @returns the transaction signature
     */
    rpc(options?: ConfirmOptions): Promise<TransactionSignature>;
    /**
     * Conveniently call both {@link rpc} and {@link pubkeys} methods.
     *
     * @param options confirmation options
     * @returns the transaction signature and account public keys
     */
    rpcAndKeys(options?: ConfirmOptions): Promise<{
        signature: TransactionSignature;
        pubkeys: InstructionAccountAddresses<IDL, I>;
    }>;
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
    prepare(): Promise<{
        instruction: TransactionInstruction;
        signers: Signer[];
        pubkeys: Partial<InstructionAccountAddresses<IDL, I>>;
    }>;
}
export {};
//# sourceMappingURL=methods.d.ts.map
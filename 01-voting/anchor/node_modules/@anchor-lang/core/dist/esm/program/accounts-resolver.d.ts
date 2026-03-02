import { PublicKey } from "@solana/web3.js";
import { Idl, IdlTypeDef } from "../idl.js";
import { AllInstructions } from "./namespace/types.js";
import Provider from "../provider.js";
import { AccountNamespace } from "./namespace/account.js";
import { PartialAccounts } from "./namespace/methods";
export type AccountsGeneric = {
    [name: string]: PublicKey | AccountsGeneric;
};
export declare function isAccountsGeneric(accounts: PublicKey | AccountsGeneric): accounts is AccountsGeneric;
export type CustomAccountResolver<IDL extends Idl> = (params: {
    args: Array<any>;
    accounts: AccountsGeneric;
    provider: Provider;
    programId: PublicKey;
    idlIx: AllInstructions<IDL>;
}) => Promise<{
    accounts: AccountsGeneric;
    resolved: number;
}>;
export declare class AccountsResolver<IDL extends Idl> {
    private _args;
    private _accounts;
    private _provider;
    private _programId;
    private _idlIx;
    private _idlTypes;
    private _customResolver?;
    private _accountStore;
    constructor(_args: any[], _accounts: AccountsGeneric, _provider: Provider, _programId: PublicKey, _idlIx: AllInstructions<IDL>, accountNamespace: AccountNamespace<IDL>, _idlTypes: IdlTypeDef[], _customResolver?: CustomAccountResolver<IDL> | undefined);
    args(args: Array<any>): void;
    resolve(): Promise<void>;
    resolveOptionals(accounts: PartialAccounts): void;
    private get;
    private set;
    private resolveOptionalsHelper;
    private resolveCustom;
    /**
     * Resolve event CPI accounts `eventAuthority` and `program`.
     *
     * Accounts will only be resolved if they are declared next to each other to
     * reduce the chance of name collision.
     */
    private resolveEventCpi;
    private resolveConst;
    private resolvePdasAndRelations;
    private parseProgramId;
    private toBuffer;
    private toBufferConst;
    private toBufferArg;
    private toBufferAccount;
    /**
     * Converts the given idl valaue into a Buffer. The values here must be
     * primitives, e.g. no structs.
     */
    private toBufferValue;
    /**
     * Recursively get the type at some path of either a primitive or a user
     * defined struct.
     */
    private getType;
}
//# sourceMappingURL=accounts-resolver.d.ts.map
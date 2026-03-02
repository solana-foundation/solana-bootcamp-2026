import { TransactionInstruction, } from "@solana/web3.js";
import { isCompositeAccounts, } from "../../idl.js";
import { IdlError } from "../../error.js";
import { toInstruction, translateAddress, validateAccounts, } from "../common.js";
import { splitArgsAndCtx } from "../context.js";
import * as features from "../../utils/features.js";
export default class InstructionNamespaceFactory {
    static build(idlIx, encodeFn, programId) {
        if (idlIx.name === "_inner") {
            throw new IdlError("the _inner name is reserved");
        }
        const ix = (...args) => {
            const [ixArgs, ctx] = splitArgsAndCtx(idlIx, [...args]);
            validateAccounts(idlIx.accounts, ctx.accounts);
            validateInstruction(idlIx, ...args);
            const keys = ix.accounts(ctx.accounts);
            if (ctx.remainingAccounts !== undefined) {
                keys.push(...ctx.remainingAccounts);
            }
            if (features.isSet("debug-logs")) {
                console.log("Outgoing account metas:", keys);
            }
            return new TransactionInstruction({
                keys,
                programId,
                data: encodeFn(idlIx.name, toInstruction(idlIx, ...ixArgs)),
            });
        };
        // Utility fn for ordering the accounts for this instruction.
        ix["accounts"] = (accs) => {
            return InstructionNamespaceFactory.accountsArray(accs, idlIx.accounts, programId, idlIx.name);
        };
        return ix;
    }
    static accountsArray(ctx, accounts, programId, ixName) {
        if (!ctx) {
            return [];
        }
        return accounts
            .map((acc) => {
            if (isCompositeAccounts(acc)) {
                const rpcAccs = ctx[acc.name];
                return InstructionNamespaceFactory.accountsArray(rpcAccs, acc.accounts, programId, ixName).flat();
            }
            let pubkey;
            try {
                pubkey = translateAddress(ctx[acc.name]);
            }
            catch (err) {
                throw new Error(`Wrong input type for account "${acc.name}" in the instruction accounts object${ixName !== undefined ? ' for instruction "' + ixName + '"' : ""}. Expected PublicKey or string.`);
            }
            const isOptional = acc.optional && pubkey.equals(programId);
            const isWritable = Boolean(acc.writable && !isOptional);
            const isSigner = Boolean(acc.signer && !isOptional);
            return {
                pubkey,
                isWritable,
                isSigner,
            };
        })
            .flat();
    }
}
// Throws error if any argument required for the `ix` is not given.
function validateInstruction(ix, ...args) {
    // todo
}
//# sourceMappingURL=instruction.js.map
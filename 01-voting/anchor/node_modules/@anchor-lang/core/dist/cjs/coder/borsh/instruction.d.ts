import { Buffer } from "buffer";
import { AccountMeta, PublicKey } from "@solana/web3.js";
import { Idl } from "../../idl.js";
import { InstructionCoder } from "../index.js";
/**
 * Encodes and decodes program instructions.
 */
export declare class BorshInstructionCoder implements InstructionCoder {
    private idl;
    private ixLayouts;
    constructor(idl: Idl);
    /**
     * Encodes a program instruction.
     */
    encode(ixName: string, ix: any): Buffer;
    /**
     * Decodes a program instruction.
     */
    decode(ix: Buffer | string, encoding?: "hex" | "base58"): Instruction | null;
    /**
     * Returns a formatted table of all the fields in the given instruction data.
     */
    format(ix: Instruction, accountMetas: AccountMeta[]): InstructionDisplay | null;
}
export type Instruction = {
    name: string;
    data: Object;
};
export type InstructionDisplay = {
    args: {
        name: string;
        type: string;
        data: string;
    }[];
    accounts: {
        name?: string;
        pubkey: PublicKey;
        isSigner: boolean;
        isWritable: boolean;
    }[];
};
//# sourceMappingURL=instruction.d.ts.map
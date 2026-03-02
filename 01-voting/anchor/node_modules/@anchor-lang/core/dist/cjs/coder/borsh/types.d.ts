import { Buffer } from "buffer";
import { Idl } from "../../idl.js";
import { TypesCoder } from "../index.js";
/**
 * Encodes and decodes user-defined types.
 */
export declare class BorshTypesCoder<N extends string = string> implements TypesCoder {
    /**
     * Maps type name to a layout.
     */
    private typeLayouts;
    constructor(idl: Idl);
    encode<T = any>(name: N, type: T): Buffer;
    decode<T = any>(name: N, data: Buffer): T;
}
//# sourceMappingURL=types.d.ts.map
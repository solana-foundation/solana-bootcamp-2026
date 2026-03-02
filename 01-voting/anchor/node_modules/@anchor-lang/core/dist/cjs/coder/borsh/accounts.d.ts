import { Buffer } from "buffer";
import { Idl } from "../../idl.js";
import { AccountsCoder } from "../index.js";
/**
 * Encodes and decodes account objects.
 */
export declare class BorshAccountsCoder<A extends string = string> implements AccountsCoder {
    private idl;
    /**
     * Maps account type identifier to a layout.
     */
    private accountLayouts;
    constructor(idl: Idl);
    encode<T = any>(accountName: A, account: T): Promise<Buffer>;
    decode<T = any>(accountName: A, data: Buffer): T;
    decodeAny<T = any>(data: Buffer): T;
    decodeUnchecked<T = any>(accountName: A, acc: Buffer): T;
    memcmp(accountName: A, appendData?: Buffer): any;
    size(accountName: A): number;
    /**
     * Get the unique discriminator prepended to all anchor accounts.
     *
     * @param name The name of the account to get the discriminator of.
     */
    accountDiscriminator(name: string): Buffer;
}
//# sourceMappingURL=accounts.d.ts.map
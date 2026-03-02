import { AccountsCoder } from "../index.js";
import { Idl } from "../../idl.js";
export declare class SystemAccountsCoder<A extends string = string> implements AccountsCoder {
    private idl;
    constructor(idl: Idl);
    encode<T = any>(accountName: A, account: T): Promise<Buffer>;
    decode<T = any>(accountName: A, ix: Buffer): T;
    decodeUnchecked<T = any>(accountName: A, ix: Buffer): T;
    memcmp(accountName: A, _appendData?: Buffer): any;
    size(accountName: A): number;
}
//# sourceMappingURL=accounts.d.ts.map
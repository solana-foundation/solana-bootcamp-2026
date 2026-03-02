import { Idl } from "../../idl.js";
import { EventCoder } from "../index.js";
export declare class BorshEventCoder implements EventCoder {
    /**
     * Maps account type identifier to a layout.
     */
    private layouts;
    constructor(idl: Idl);
    decode(log: string): {
        name: string;
        data: any;
    } | null;
}
//# sourceMappingURL=event.d.ts.map
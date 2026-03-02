import { Buffer } from "buffer";
import * as base64 from "../../utils/bytes/base64.js";
import { IdlCoder } from "./idl.js";
export class BorshEventCoder {
    constructor(idl) {
        if (!idl.events) {
            this.layouts = new Map();
            return;
        }
        const types = idl.types;
        if (!types) {
            throw new Error("Events require `idl.types`");
        }
        const layouts = idl.events.map((ev) => {
            const typeDef = types.find((ty) => ty.name === ev.name);
            if (!typeDef) {
                throw new Error(`Event not found: ${ev.name}`);
            }
            return [
                ev.name,
                {
                    discriminator: ev.discriminator,
                    layout: IdlCoder.typeDefLayout({ typeDef, types }),
                },
            ];
        });
        this.layouts = new Map(layouts);
    }
    decode(log) {
        let logArr;
        // This will throw if log length is not a multiple of 4.
        try {
            logArr = base64.decode(log);
        }
        catch (e) {
            return null;
        }
        for (const [name, layout] of this.layouts) {
            const givenDisc = logArr.subarray(0, layout.discriminator.length);
            const matches = givenDisc.equals(Buffer.from(layout.discriminator));
            if (matches) {
                return {
                    name,
                    data: layout.layout.decode(logArr.subarray(givenDisc.length)),
                };
            }
        }
        return null;
    }
}
//# sourceMappingURL=event.js.map
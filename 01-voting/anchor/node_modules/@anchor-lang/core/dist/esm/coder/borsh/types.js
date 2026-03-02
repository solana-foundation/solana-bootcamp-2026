import { Buffer } from "buffer";
import { IdlCoder } from "./idl.js";
/**
 * Encodes and decodes user-defined types.
 */
export class BorshTypesCoder {
    constructor(idl) {
        const types = idl.types;
        if (!types) {
            this.typeLayouts = new Map();
            return;
        }
        const layouts = types
            .filter((ty) => !ty.generics)
            .map((ty) => [
            ty.name,
            IdlCoder.typeDefLayout({ typeDef: ty, types }),
        ]);
        this.typeLayouts = new Map(layouts);
    }
    encode(name, type) {
        const buffer = Buffer.alloc(1000); // TODO: use a tighter buffer.
        const layout = this.typeLayouts.get(name);
        if (!layout) {
            throw new Error(`Unknown type: ${name}`);
        }
        const len = layout.encode(type, buffer);
        return buffer.slice(0, len);
    }
    decode(name, data) {
        const layout = this.typeLayouts.get(name);
        if (!layout) {
            throw new Error(`Unknown type: ${name}`);
        }
        return layout.decode(data);
    }
}
//# sourceMappingURL=types.js.map
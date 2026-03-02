"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BorshTypesCoder = void 0;
const buffer_1 = require("buffer");
const idl_js_1 = require("./idl.js");
/**
 * Encodes and decodes user-defined types.
 */
class BorshTypesCoder {
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
            idl_js_1.IdlCoder.typeDefLayout({ typeDef: ty, types }),
        ]);
        this.typeLayouts = new Map(layouts);
    }
    encode(name, type) {
        const buffer = buffer_1.Buffer.alloc(1000); // TODO: use a tighter buffer.
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
exports.BorshTypesCoder = BorshTypesCoder;
//# sourceMappingURL=types.js.map
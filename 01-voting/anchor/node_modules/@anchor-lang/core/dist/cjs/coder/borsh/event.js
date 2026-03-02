"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.BorshEventCoder = void 0;
const buffer_1 = require("buffer");
const base64 = __importStar(require("../../utils/bytes/base64.js"));
const idl_js_1 = require("./idl.js");
class BorshEventCoder {
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
                    layout: idl_js_1.IdlCoder.typeDefLayout({ typeDef, types }),
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
            const matches = givenDisc.equals(buffer_1.Buffer.from(layout.discriminator));
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
exports.BorshEventCoder = BorshEventCoder;
//# sourceMappingURL=event.js.map
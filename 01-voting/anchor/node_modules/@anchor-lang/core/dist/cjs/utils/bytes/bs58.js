"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.encode = encode;
exports.decode = decode;
const bs58_1 = __importDefault(require("bs58"));
function encode(data) {
    return bs58_1.default.encode(data);
}
function decode(data) {
    return bs58_1.default.decode(data);
}
//# sourceMappingURL=bs58.js.map
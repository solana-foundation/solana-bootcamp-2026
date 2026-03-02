"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encode = encode;
exports.decode = decode;
const buffer_1 = require("buffer");
function encode(data) {
    return data.toString("base64");
}
function decode(data) {
    return buffer_1.Buffer.from(data, "base64");
}
//# sourceMappingURL=base64.js.map
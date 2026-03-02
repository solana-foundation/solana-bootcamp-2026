"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.hash = hash;
const sha256_1 = require("@noble/hashes/sha256");
function hash(data) {
    return new TextDecoder().decode((0, sha256_1.sha256)(data));
}
//# sourceMappingURL=sha256.js.map
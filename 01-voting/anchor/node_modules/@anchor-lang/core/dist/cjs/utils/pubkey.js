"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createWithSeedSync = createWithSeedSync;
const buffer_1 = require("buffer");
const web3_js_1 = require("@solana/web3.js");
const sha256_1 = require("@noble/hashes/sha256");
// Sync version of web3.PublicKey.createWithSeed.
function createWithSeedSync(fromPublicKey, seed, programId) {
    const buffer = buffer_1.Buffer.concat([
        fromPublicKey.toBuffer(),
        buffer_1.Buffer.from(seed),
        programId.toBuffer(),
    ]);
    return new web3_js_1.PublicKey((0, sha256_1.sha256)(buffer));
}
//# sourceMappingURL=pubkey.js.map
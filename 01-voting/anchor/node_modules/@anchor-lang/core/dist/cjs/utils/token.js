"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ASSOCIATED_PROGRAM_ID = exports.TOKEN_PROGRAM_ID = void 0;
exports.associatedAddress = associatedAddress;
const web3_js_1 = require("@solana/web3.js");
exports.TOKEN_PROGRAM_ID = new web3_js_1.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
exports.ASSOCIATED_PROGRAM_ID = new web3_js_1.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
function associatedAddress({ mint, owner, }) {
    return web3_js_1.PublicKey.findProgramAddressSync([owner.toBuffer(), exports.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()], exports.ASSOCIATED_PROGRAM_ID)[0];
}
//# sourceMappingURL=token.js.map
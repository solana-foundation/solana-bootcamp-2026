"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.set = set;
exports.isSet = isSet;
const _AVAILABLE_FEATURES = new Set(["debug-logs"]);
const _FEATURES = new Map();
function set(key) {
    if (!_AVAILABLE_FEATURES.has(key)) {
        throw new Error("Invalid feature");
    }
    _FEATURES.set(key, true);
}
function isSet(key) {
    return _FEATURES.get(key) !== undefined;
}
//# sourceMappingURL=features.js.map
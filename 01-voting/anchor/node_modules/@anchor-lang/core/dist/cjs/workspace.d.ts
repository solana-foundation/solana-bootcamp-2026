import { Program } from "./program/index.js";
import { Idl } from "./idl.js";
/**
 * The `workspace` namespace provides a convenience API to automatically
 * search for and deserialize [[Program]] objects defined by compiled IDLs
 * in an Anchor workspace.
 *
 * This API is for Node only.
 */
declare const workspace: {
    [key: string]: Program<Idl>;
};
export default workspace;
//# sourceMappingURL=workspace.d.ts.map
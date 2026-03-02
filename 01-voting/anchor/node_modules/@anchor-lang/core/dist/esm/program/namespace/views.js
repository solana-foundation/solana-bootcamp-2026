import { IdlCoder } from "../../coder/borsh/idl";
import { decode } from "../../utils/bytes/base64";
export default class ViewFactory {
    static build(programId, idlIx, simulateFn, idl) {
        const isWritable = idlIx.accounts.find((a) => a.writable);
        const hasReturn = !!idlIx.returns;
        if (isWritable || !hasReturn)
            return;
        const view = async (...args) => {
            let simulationResult = await simulateFn(...args);
            const returnPrefix = `Program return: ${programId} `;
            let returnLog = simulationResult.raw.find((l) => l.startsWith(returnPrefix));
            if (!returnLog) {
                throw new Error("View expected return log");
            }
            let returnData = decode(returnLog.slice(returnPrefix.length));
            let returnType = idlIx.returns;
            if (!returnType) {
                throw new Error("View expected return type");
            }
            const coder = IdlCoder.fieldLayout({ type: returnType }, idl.types);
            return coder.decode(returnData);
        };
        return view;
    }
}
//# sourceMappingURL=views.js.map
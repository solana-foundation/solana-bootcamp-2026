import { Layout } from "buffer-layout";
import { IdlField, IdlTypeDef, IdlType, IdlGenericArg, Idl } from "../../idl.js";
type PartialField = {
    name?: string;
} & Pick<IdlField, "type">;
export declare class IdlCoder {
    static fieldLayout(field: PartialField, types?: IdlTypeDef[], genericArgs?: IdlGenericArg[] | null): Layout;
    /**
     * Get the type layout of the given defined type(struct or enum).
     */
    static typeDefLayout({ typeDef, types, name, genericArgs, }: {
        typeDef: IdlTypeDef;
        types: IdlTypeDef[];
        genericArgs?: IdlGenericArg[] | null;
        name?: string;
    }): Layout;
    /**
     * Get the type of the size in bytes. Returns `1` for variable length types.
     */
    static typeSize(ty: IdlType, idl: Idl, genericArgs?: IdlGenericArg[] | null): number;
    /**
     * Resolve the generic array length or return the constant-sized array length.
     */
    private static resolveArrayLen;
    /**
     * Recursively resolve generic arguments i.e. replace all generics with the
     * actual type that they hold based on the initial `genericArgs` given.
     */
    private static resolveGenericArgs;
}
export {};
//# sourceMappingURL=idl.d.ts.map
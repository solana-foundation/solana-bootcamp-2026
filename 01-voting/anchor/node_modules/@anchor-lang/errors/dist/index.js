"use strict";
// Instruction errors.
Object.defineProperty(exports, "__esModule", { value: true });
exports.ANCHOR_ERROR__REQUIRE_EQ_VIOLATED = exports.ANCHOR_ERROR__REQUIRE_VIOLATED = exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION_PROGRAM_ID = exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_MINT_PERMANENT_DELEGATE_EXTENSION_DELEGATE = exports.ANCHOR_ERROR__CONSTRAINT_MINT_PERMANENT_DELEGATE_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_MINT_CLOSE_AUTHORITY_EXTENSION_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_CLOSE_AUTHORITY_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION_METADATA_ADDRESS = exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION_MEMBER_ADDRESS = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION_GROUP_ADDRESS = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION = exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED_TOKEN_TOKEN_PROGRAM = exports.ANCHOR_ERROR__CONSTRAINT_MINT_TOKEN_PROGRAM = exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_TOKEN_PROGRAM = exports.ANCHOR_ERROR__CONSTRAINT_ACCOUNT_IS_NONE = exports.ANCHOR_ERROR__CONSTRAINT_SPACE = exports.ANCHOR_ERROR__CONSTRAINT_MINT_DECIMALS = exports.ANCHOR_ERROR__CONSTRAINT_MINT_FREEZE_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_MINT_MINT_AUTHORITY = exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_OWNER = exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_MINT = exports.ANCHOR_ERROR__CONSTRAINT_ZERO = exports.ANCHOR_ERROR__CONSTRAINT_ADDRESS = exports.ANCHOR_ERROR__CONSTRAINT_CLOSE = exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED_INIT = exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED = exports.ANCHOR_ERROR__CONSTRAINT_STATE = exports.ANCHOR_ERROR__CONSTRAINT_EXECUTABLE = exports.ANCHOR_ERROR__CONSTRAINT_SEEDS = exports.ANCHOR_ERROR__CONSTRAINT_RENT_EXEMPT = exports.ANCHOR_ERROR__CONSTRAINT_OWNER = exports.ANCHOR_ERROR__CONSTRAINT_RAW = exports.ANCHOR_ERROR__CONSTRAINT_SIGNER = exports.ANCHOR_ERROR__CONSTRAINT_HAS_ONE = exports.ANCHOR_ERROR__CONSTRAINT_MUT = exports.ANCHOR_ERROR__EVENT_INSTRUCTION_STUB = exports.ANCHOR_ERROR__IDL_ACCOUNT_NOT_EMPTY = exports.ANCHOR_ERROR__IDL_INSTRUCTION_INVALID_PROGRAM = exports.ANCHOR_ERROR__IDL_INSTRUCTION_STUB = exports.ANCHOR_ERROR__INSTRUCTION_DID_NOT_SERIALIZE = exports.ANCHOR_ERROR__INSTRUCTION_DID_NOT_DESERIALIZE = exports.ANCHOR_ERROR__INSTRUCTION_FALLBACK_NOT_FOUND = exports.ANCHOR_ERROR__INSTRUCTION_MISSING = void 0;
exports.ANCHOR_ERROR__DEPRECATED = exports.ANCHOR_ERROR__INVALID_NUMERIC_CONVERSION = exports.ANCHOR_ERROR__TRYING_TO_INIT_PAYER_AS_PROGRAM_ACCOUNT = exports.ANCHOR_ERROR__DECLARED_PROGRAM_ID_MISMATCH = exports.ANCHOR_ERROR__ACCOUNT_DUPLICATE_REALLOCS = exports.ANCHOR_ERROR__ACCOUNT_REALLOC_EXCEEDS_LIMIT = exports.ANCHOR_ERROR__ACCOUNT_SYSVAR_MISMATCH = exports.ANCHOR_ERROR__ACCOUNT_NOT_ASSOCIATED_TOKEN_ACCOUNT = exports.ANCHOR_ERROR__ACCOUNT_NOT_PROGRAM_DATA = exports.ANCHOR_ERROR__ACCOUNT_NOT_INITIALIZED = exports.ANCHOR_ERROR__ACCOUNT_NOT_SYSTEM_OWNED = exports.ANCHOR_ERROR__ACCOUNT_NOT_SIGNER = exports.ANCHOR_ERROR__INVALID_PROGRAM_EXECUTABLE = exports.ANCHOR_ERROR__INVALID_PROGRAM_ID = exports.ANCHOR_ERROR__ACCOUNT_OWNED_BY_WRONG_PROGRAM = exports.ANCHOR_ERROR__ACCOUNT_NOT_MUTABLE = exports.ANCHOR_ERROR__ACCOUNT_NOT_ENOUGH_KEYS = exports.ANCHOR_ERROR__ACCOUNT_DID_NOT_SERIALIZE = exports.ANCHOR_ERROR__ACCOUNT_DID_NOT_DESERIALIZE = exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_MISMATCH = exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_NOT_FOUND = exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_ALREADY_SET = exports.ANCHOR_ERROR__REQUIRE_GTE_VIOLATED = exports.ANCHOR_ERROR__REQUIRE_GT_VIOLATED = exports.ANCHOR_ERROR__REQUIRE_KEYS_NEQ_VIOLATED = exports.ANCHOR_ERROR__REQUIRE_NEQ_VIOLATED = exports.ANCHOR_ERROR__REQUIRE_KEYS_EQ_VIOLATED = void 0;
/** 8 byte instruction identifier not provided. */
exports.ANCHOR_ERROR__INSTRUCTION_MISSING = 100;
/** Fallback functions are not supported. */
exports.ANCHOR_ERROR__INSTRUCTION_FALLBACK_NOT_FOUND = 101;
/** The program could not deserialize the given instruction. */
exports.ANCHOR_ERROR__INSTRUCTION_DID_NOT_DESERIALIZE = 102;
/** The program could not serialize the given instruction. */
exports.ANCHOR_ERROR__INSTRUCTION_DID_NOT_SERIALIZE = 103;
// IDL instruction errors.
/** The program was compiled without idl instructions. */
exports.ANCHOR_ERROR__IDL_INSTRUCTION_STUB = 1000;
/** The transaction was given an invalid program for the IDL instruction. */
exports.ANCHOR_ERROR__IDL_INSTRUCTION_INVALID_PROGRAM = 1001;
/** IDL account must be empty in order to resize, try closing first. */
exports.ANCHOR_ERROR__IDL_ACCOUNT_NOT_EMPTY = 1002;
// Event instructions.
/** The program was compiled without `event-cpi` feature. */
exports.ANCHOR_ERROR__EVENT_INSTRUCTION_STUB = 1500;
// Constraint errors.
/** A mut constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MUT = 2000;
/** A has one constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_HAS_ONE = 2001;
/** A signer constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_SIGNER = 2002;
/** A raw constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_RAW = 2003;
/** An owner constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_OWNER = 2004;
/** A rent exemption constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_RENT_EXEMPT = 2005;
/** A seeds constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_SEEDS = 2006;
/** An executable constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_EXECUTABLE = 2007;
/** Deprecated Error, feel free to replace with something else. */
exports.ANCHOR_ERROR__CONSTRAINT_STATE = 2008;
/** An associated constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED = 2009;
/** An associated init constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED_INIT = 2010;
/** A close constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_CLOSE = 2011;
/** An address constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_ADDRESS = 2012;
/** Expected zero account discriminant. */
exports.ANCHOR_ERROR__CONSTRAINT_ZERO = 2013;
/** A token mint constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_MINT = 2014;
/** A token owner constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_OWNER = 2015;
/** A mint mint authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_MINT_AUTHORITY = 2016;
/** A mint freeze authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_FREEZE_AUTHORITY = 2017;
/** A mint decimals constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_DECIMALS = 2018;
/** A space constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_SPACE = 2019;
/** A required account for the constraint is None. */
exports.ANCHOR_ERROR__CONSTRAINT_ACCOUNT_IS_NONE = 2020;
/** A token account token program constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_TOKEN_TOKEN_PROGRAM = 2021;
/** A mint token program constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_TOKEN_PROGRAM = 2022;
/** An associated token account token program constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_ASSOCIATED_TOKEN_TOKEN_PROGRAM = 2023;
/** A group pointer extension constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION = 2024;
/** A group pointer extension authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION_AUTHORITY = 2025;
/** A group pointer extension group address constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_POINTER_EXTENSION_GROUP_ADDRESS = 2026;
/** A group member pointer extension constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION = 2027;
/** A group member pointer extension authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION_AUTHORITY = 2028;
/** A group member pointer extension group address constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_GROUP_MEMBER_POINTER_EXTENSION_MEMBER_ADDRESS = 2029;
/** A metadata pointer extension constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION = 2030;
/** A metadata pointer extension authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION_AUTHORITY = 2031;
/** A metadata pointer extension metadata address constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_METADATA_POINTER_EXTENSION_METADATA_ADDRESS = 2032;
/** A close authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_CLOSE_AUTHORITY_EXTENSION = 2033;
/** A close authority extension authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_CLOSE_AUTHORITY_EXTENSION_AUTHORITY = 2034;
/** A permanent delegate extension constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_PERMANENT_DELEGATE_EXTENSION = 2035;
/** A permanent delegate extension delegate constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_PERMANENT_DELEGATE_EXTENSION_DELEGATE = 2036;
/** A transfer hook extension constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION = 2037;
/** A transfer hook extension authority constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION_AUTHORITY = 2038;
/** A transfer hook extension transfer hook program id constraint was violated. */
exports.ANCHOR_ERROR__CONSTRAINT_MINT_TRANSFER_HOOK_EXTENSION_PROGRAM_ID = 2039;
// Require errors.
/** A require expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_VIOLATED = 2500;
/** A require_eq expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_EQ_VIOLATED = 2501;
/** A require_keys_eq expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_KEYS_EQ_VIOLATED = 2502;
/** A require_neq expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_NEQ_VIOLATED = 2503;
/** A require_keys_neq expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_KEYS_NEQ_VIOLATED = 2504;
/** A require_gt expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_GT_VIOLATED = 2505;
/** A require_gte expression was violated. */
exports.ANCHOR_ERROR__REQUIRE_GTE_VIOLATED = 2506;
// Account errors.
/** The account discriminator was already set on this account. */
exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_ALREADY_SET = 3000;
/** No 8 byte discriminator was found on the account. */
exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_NOT_FOUND = 3001;
/** 8 byte discriminator did not match what was expected. */
exports.ANCHOR_ERROR__ACCOUNT_DISCRIMINATOR_MISMATCH = 3002;
/** Failed to deserialize the account. */
exports.ANCHOR_ERROR__ACCOUNT_DID_NOT_DESERIALIZE = 3003;
/** Failed to serialize the account. */
exports.ANCHOR_ERROR__ACCOUNT_DID_NOT_SERIALIZE = 3004;
/** Not enough account keys given to the instruction. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_ENOUGH_KEYS = 3005;
/** The given account is not mutable. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_MUTABLE = 3006;
/** The given account is owned by a different program than expected. */
exports.ANCHOR_ERROR__ACCOUNT_OWNED_BY_WRONG_PROGRAM = 3007;
/** Program ID was not as expected. */
exports.ANCHOR_ERROR__INVALID_PROGRAM_ID = 3008;
/** Program account is not executable. */
exports.ANCHOR_ERROR__INVALID_PROGRAM_EXECUTABLE = 3009;
/** The given account did not sign. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_SIGNER = 3010;
/** The given account is not owned by the system program. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_SYSTEM_OWNED = 3011;
/** The program expected this account to be already initialized. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_INITIALIZED = 3012;
/** The given account is not a program data account. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_PROGRAM_DATA = 3013;
/** The given account is not the associated token account. */
exports.ANCHOR_ERROR__ACCOUNT_NOT_ASSOCIATED_TOKEN_ACCOUNT = 3014;
/** The given public key does not match the required sysvar. */
exports.ANCHOR_ERROR__ACCOUNT_SYSVAR_MISMATCH = 3015;
/** The account reallocation exceeds the MAX_PERMITTED_DATA_INCREASE limit. */
exports.ANCHOR_ERROR__ACCOUNT_REALLOC_EXCEEDS_LIMIT = 3016;
/** The account was duplicated for more than one reallocation. */
exports.ANCHOR_ERROR__ACCOUNT_DUPLICATE_REALLOCS = 3017;
// Miscellaneous errors.
/** The declared program id does not match the actual program id. */
exports.ANCHOR_ERROR__DECLARED_PROGRAM_ID_MISMATCH = 4100;
/** You cannot/should not initialize the payer account as a program account. */
exports.ANCHOR_ERROR__TRYING_TO_INIT_PAYER_AS_PROGRAM_ACCOUNT = 4101;
/** The program could not perform the numeric conversion, out of range integral type conversion attempted. */
exports.ANCHOR_ERROR__INVALID_NUMERIC_CONVERSION = 4102;
// Deprecated errors.
/** The API being used is deprecated and should no longer be used. */
exports.ANCHOR_ERROR__DEPRECATED = 5000;
//# sourceMappingURL=index.js.map
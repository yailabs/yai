#ifndef YAI_SDK_ERRORS_H
#define YAI_SDK_ERRORS_H

/**
 * @file errors.h
 * @brief Stable SDK error and return-code classes.
 *
 * Exit-code classes are aligned with YAI CLI/operator semantics.
 */

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Stable SDK error classes.
 */
typedef enum
{
    /** Success. */
    YAI_SDK_OK = 0,

    /** Invalid arguments / contract violation. */
    YAI_SDK_BAD_ARGS = 2,

    /** Required dependency missing (registry, law artifacts, etc.). */
    YAI_SDK_DEPS_MISSING = 3,

    /** Runtime handshake failed or runtime not ready. */
    YAI_SDK_RUNTIME_NOT_READY = 4,

    /** Authority/role gate rejected request. */
    YAI_SDK_UNAUTHORIZED = 5,

    /** Command registered but not implemented. */
    YAI_SDK_NYI = 6,

    /** Local I/O failure. */
    YAI_SDK_IO = 10,

    /** RPC transport failure. */
    YAI_SDK_RPC = 20,

    /** Runtime endpoint/locator cannot be resolved. */
    YAI_SDK_ENDPOINT_UNRESOLVED = 21,

    /** Runtime endpoint target is invalid. */
    YAI_SDK_ENDPOINT_INVALID = 22,

    /** Runtime transport kind is unsupported in current SDK build. */
    YAI_SDK_TRANSPORT_UNSUPPORTED = 23,

    /** Protocol envelope/parsing failure. */
    YAI_SDK_PROTOCOL = 30,

    /** Runtime endpoint unavailable (ENOTCONN-compatible class). */
    YAI_SDK_SERVER_OFF = 107,

    /** Generic not implemented class for compatibility. */
    YAI_SDK_NOT_IMPLEMENTED = 99
} yai_sdk_err_t;

#ifndef YAI_SDK_ERR_USAGE
#define YAI_SDK_ERR_USAGE YAI_SDK_BAD_ARGS
#endif

/**
 * @brief Return short stable token for an SDK error class.
 */
static inline const char *yai_sdk_err_str(yai_sdk_err_t code)
{
    switch (code)
    {
    case YAI_SDK_OK:
        return "ok";
    case YAI_SDK_BAD_ARGS:
        return "bad_args";
    case YAI_SDK_DEPS_MISSING:
        return "deps_missing";
    case YAI_SDK_RUNTIME_NOT_READY:
        return "runtime_not_ready";
    case YAI_SDK_UNAUTHORIZED:
        return "unauthorized";
    case YAI_SDK_NYI:
        return "nyi";
    case YAI_SDK_IO:
        return "io";
    case YAI_SDK_RPC:
        return "rpc";
    case YAI_SDK_ENDPOINT_UNRESOLVED:
        return "endpoint_unresolved";
    case YAI_SDK_ENDPOINT_INVALID:
        return "endpoint_invalid";
    case YAI_SDK_TRANSPORT_UNSUPPORTED:
        return "transport_unsupported";
    case YAI_SDK_PROTOCOL:
        return "protocol";
    case YAI_SDK_SERVER_OFF:
        return "server_off";
    case YAI_SDK_NOT_IMPLEMENTED:
        return "not_implemented";
    default:
        return "unknown";
    }
}

/**
 * @brief Stable ABI symbol for error token conversion.
 */
const char *yai_sdk_errstr(yai_sdk_err_t code);

#ifdef __cplusplus
}
#endif

#endif

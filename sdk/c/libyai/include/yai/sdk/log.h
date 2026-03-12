/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

/** Log levels for SDK-internal diagnostics callbacks. */
typedef enum yai_sdk_log_level {
    YAI_SDK_LOG_DEBUG = 0,
    YAI_SDK_LOG_INFO = 1,
    YAI_SDK_LOG_WARN = 2,
    YAI_SDK_LOG_ERROR = 3,
} yai_sdk_log_level_t;

/** User-provided log callback. */
typedef void (*yai_sdk_log_handler_t)(
    yai_sdk_log_level_t level,
    const char *component,
    const char *message,
    void *user_data);

/**
 * Register a log handler. Pass NULL to disable SDK logging callbacks.
 */
void yai_sdk_set_log_handler(yai_sdk_log_handler_t handler, void *user_data);

/** Set minimum log level delivered to the handler. */
void yai_sdk_set_log_level(yai_sdk_log_level_t level);

/** Get current minimum log level. */
yai_sdk_log_level_t yai_sdk_get_log_level(void);

#ifdef __cplusplus
}
#endif

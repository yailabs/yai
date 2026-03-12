/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/sdk/log.h>

#include <stddef.h>

static yai_sdk_log_handler_t g_log_handler = NULL;
static void *g_log_user_data = NULL;
static yai_sdk_log_level_t g_min_log_level = YAI_SDK_LOG_WARN;

void yai_sdk_set_log_handler(yai_sdk_log_handler_t handler, void *user_data)
{
    g_log_handler = handler;
    g_log_user_data = user_data;
}

void yai_sdk_set_log_level(yai_sdk_log_level_t level)
{
    g_min_log_level = level;
}

yai_sdk_log_level_t yai_sdk_get_log_level(void)
{
    return g_min_log_level;
}

void yai_sdk_log_emit(yai_sdk_log_level_t level, const char *component, const char *message)
{
    if (!g_log_handler) return;
    if (level < g_min_log_level) return;
    g_log_handler(level,
                  (component && component[0]) ? component : "sdk",
                  (message && message[0]) ? message : "",
                  g_log_user_data);
}

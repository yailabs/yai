/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/sdk/log.h>

void yai_sdk_log_emit(yai_sdk_log_level_t level, const char *component, const char *message);

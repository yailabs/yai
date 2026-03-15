/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/cognition/errors.h>
#include <yai/cognition/types.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_config {
  const char *runtime_name;
  int enable_mock_provider;
} yai_config_t;

typedef struct yai_runtime {
  int initialized;
  int transport_ready;
  int providers_ready;
  int memory_ready;
  int cognition_ready;
  yai_config_t config;
} yai_runtime_t;

/*
 * Transitional compatibility surface.
 * Canonical lifecycle ownership is in core/lifecycle runtime capabilities.
 */
int yai_init(const yai_config_t *config);
int yai_shutdown(void);
int yai_is_initialized(void);
const yai_runtime_t *yai_runtime_state(void);

#ifdef __cplusplus
}
#endif

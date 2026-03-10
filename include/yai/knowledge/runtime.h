/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/knowledge/errors.h>
#include <yai/knowledge/types.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_mind_config {
  const char *runtime_name;
  int enable_mock_provider;
} yai_mind_config_t;

typedef struct yai_mind_runtime {
  int initialized;
  int transport_ready;
  int providers_ready;
  int memory_ready;
  int cognition_ready;
  yai_mind_config_t config;
} yai_mind_runtime_t;

/*
 * Transitional compatibility surface.
 * Canonical lifecycle ownership is in core/lifecycle runtime capabilities.
 */
int yai_mind_init(const yai_mind_config_t *config);
int yai_mind_shutdown(void);
int yai_mind_is_initialized(void);
const yai_mind_runtime_t *yai_mind_runtime_state(void);

#ifdef __cplusplus
}
#endif

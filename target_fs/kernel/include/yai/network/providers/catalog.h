#pragma once

/*
 * Transitional compatibility layer for old orchestration provider API.
 * Canonical provider runtime lives elsewhere; this shim exists only
 * to keep the current orchestration stack compiling during cutover.
 */

#include <stddef.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

#include <yai/fabric/providers/registry.h>

typedef struct yai_provider_config {
  char id[32];
  char host[128];
  int port;
  char endpoint[128];
} yai_provider_config_t;

static inline int yai_provider_gate_init(const yai_provider_config_t *cfg)
{
  (void)cfg;
  return 0;
}

static inline int yai_client_completion(const char *provider_id,
                                        const char *prompt,
                                        yai_provider_response_t *response_out)
{
  (void)provider_id;
  (void)prompt;

  if (!response_out) return -1;
  memset(response_out, 0, sizeof(*response_out));
  snprintf(response_out->output, sizeof(response_out->output),
           "%s", "provider compatibility shim: completion unavailable");
  return -1;
}

static inline int yai_client_embedding(const char *provider_id,
                                       const char *text,
                                       float *vector_out,
                                       size_t vector_cap)
{
  size_t i;

  (void)provider_id;
  (void)text;

  if (!vector_out || vector_cap == 0) return -1;
  for (i = 0; i < vector_cap; ++i) vector_out[i] = 0.0f;
  return -1;
}

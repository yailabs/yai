/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/knowledge/errors.h>
#include <yai/knowledge/types.h>

#ifdef __cplusplus
extern "C" {
#endif

struct yai_mind_provider;

typedef struct yai_mind_provider_vtable {
  int (*completion)(struct yai_mind_provider *provider,
                    const yai_mind_provider_request_t *request,
                    yai_mind_provider_response_t *response);
  int (*embedding)(struct yai_mind_provider *provider,
                   const char *text,
                   float *vector_out,
                   size_t vector_dim);
  void (*destroy)(struct yai_mind_provider *provider);
} yai_mind_provider_vtable_t;

typedef struct yai_mind_provider {
  const char *name;
  void *state;
  const yai_mind_provider_vtable_t *vtable;
} yai_mind_provider_t;

typedef struct yai_mind_provider_registry {
  yai_mind_provider_t *providers[16];
  size_t count;
  yai_mind_provider_t *default_provider;
} yai_mind_provider_registry_t;

int yai_mind_providers_init(void);
int yai_mind_providers_shutdown(void);
yai_mind_provider_registry_t *yai_mind_providers_registry(void);
/* Canonical knowledge-owned provider lifecycle entrypoints. */
int yai_knowledge_providers_start(void);
int yai_knowledge_providers_stop(void);
yai_mind_provider_registry_t *yai_knowledge_provider_registry(void);

int yai_mind_provider_registry_init(yai_mind_provider_registry_t *registry);
int yai_mind_provider_registry_register(yai_mind_provider_registry_t *registry,
                                        yai_mind_provider_t *provider,
                                        int make_default);
yai_mind_provider_t *yai_mind_provider_registry_get(yai_mind_provider_registry_t *registry,
                                                     const char *name);
yai_mind_provider_t *yai_mind_provider_registry_default(yai_mind_provider_registry_t *registry);
void yai_mind_provider_registry_shutdown(yai_mind_provider_registry_t *registry);

int yai_mind_provider_completion(yai_mind_provider_t *provider,
                                 const yai_mind_provider_request_t *request,
                                 yai_mind_provider_response_t *response);

int yai_mind_client_completion(const char *provider_name,
                               const char *payload,
                               yai_mind_provider_response_t *response_out);
int yai_mind_client_embedding(const char *provider_name,
                              const char *text,
                              float *vector_out,
                              size_t vector_dim);

int yai_mind_mock_provider_create(yai_mind_provider_t **provider_out);

#ifdef __cplusplus
}
#endif

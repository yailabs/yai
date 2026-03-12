/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#include <yai/knowledge/errors.h>
#include <yai/knowledge/types.h>

#ifdef __cplusplus
extern "C" {
#endif

enum { YAI_PROVIDER_REGISTRY_CAPACITY = 16 };

typedef enum yai_provider_capability {
  YAI_PROVIDER_CAPABILITY_EMBEDDING = 1u << 0,
  YAI_PROVIDER_CAPABILITY_INFERENCE = 1u << 1
} yai_provider_capability_t;

typedef enum yai_provider_class {
  YAI_PROVIDER_CLASS_UNKNOWN = 0,
  YAI_PROVIDER_CLASS_EMBEDDING = 1,
  YAI_PROVIDER_CLASS_INFERENCE = 2,
  YAI_PROVIDER_CLASS_HYBRID = 3
} yai_provider_class_t;

typedef enum yai_provider_trust_level {
  YAI_PROVIDER_TRUST_UNTRUSTED = 0,
  YAI_PROVIDER_TRUST_SANDBOXED = 1,
  YAI_PROVIDER_TRUST_TRUSTED = 2
} yai_provider_trust_level_t;

typedef struct yai_provider_descriptor {
  const char *provider_id;
  yai_provider_class_t provider_class;
  unsigned int capability_mask;
  yai_provider_trust_level_t trust_level;
  int is_mock;
  const char *invocation_mode;
} yai_provider_descriptor_t;

struct yai_provider;

typedef struct yai_provider_vtable {
  int (*completion)(struct yai_provider *provider,
                    const yai_provider_request_t *request,
                    yai_provider_response_t *response);
  int (*embedding)(struct yai_provider *provider,
                   const char *text,
                   float *vector_out,
                   size_t vector_dim);
  void (*destroy)(struct yai_provider *provider);
} yai_provider_vtable_t;

typedef struct yai_provider {
  const char *name;
  void *state;
  const yai_provider_vtable_t *vtable;
} yai_provider_t;

typedef struct yai_provider_registry {
  yai_provider_t *providers[YAI_PROVIDER_REGISTRY_CAPACITY];
  yai_provider_descriptor_t descriptors[YAI_PROVIDER_REGISTRY_CAPACITY];
  size_t count;
  yai_provider_t *default_provider;
} yai_provider_registry_t;

int yai_provider_registry_init(yai_provider_registry_t *registry);
int yai_provider_registry_register_with_descriptor(yai_provider_registry_t *registry,
                                                   yai_provider_t *provider,
                                                   const yai_provider_descriptor_t *descriptor,
                                                   int make_default);
int yai_provider_registry_register(yai_provider_registry_t *registry,
                                   yai_provider_t *provider,
                                   int make_default);
yai_provider_t *yai_provider_registry_get(yai_provider_registry_t *registry,
                                          const char *name);
const yai_provider_descriptor_t *yai_provider_registry_get_descriptor(yai_provider_registry_t *registry,
                                                                      const char *name);
const yai_provider_descriptor_t *yai_provider_registry_descriptor_for(yai_provider_registry_t *registry,
                                                                      const yai_provider_t *provider);
yai_provider_t *yai_provider_registry_default(yai_provider_registry_t *registry);
void yai_provider_registry_shutdown(yai_provider_registry_t *registry);

int yai_provider_completion(yai_provider_t *provider,
                            const yai_provider_request_t *request,
                            yai_provider_response_t *response);

int yai_providers_init(void);
int yai_providers_shutdown(void);
yai_provider_registry_t *yai_providers_registry(void);
int yai_knowledge_providers_start(void);
int yai_knowledge_providers_stop(void);
yai_provider_registry_t *yai_knowledge_provider_registry(void);

#ifdef __cplusplus
}
#endif

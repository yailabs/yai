#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_TRACE_SOURCE_NAME_MAX 64u
#define YAI_TRACE_SOURCE_MAX 128u

typedef struct {
  uint64_t source_handle;
  uint64_t owner_handle;
  yai_trace_domain_t domain;
  char name[YAI_TRACE_SOURCE_NAME_MAX];
} yai_trace_source_t;

void yai_trace_registry_bootstrap(void);
int yai_trace_source_register(yai_trace_domain_t domain,
                              uint64_t owner_handle,
                              const char *name,
                              yai_trace_source_t *out_source);
int yai_trace_source_list(yai_trace_source_t *entries, size_t cap, size_t *out_len);

#ifdef __cplusplus
}
#endif

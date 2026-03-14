#pragma once

#include <stddef.h>

#include <yai/proc/process.h>

#ifdef __cplusplus
extern "C" {
#endif

#define YAI_PROCESS_REGISTRY_MAX 256u

typedef struct {
  yai_process_t entries[YAI_PROCESS_REGISTRY_MAX];
  size_t len;
} yai_process_registry_t;

void yai_process_registry_bootstrap(void);
int yai_process_registry_register(const yai_process_t *process);
int yai_process_registry_get(yai_process_handle_t handle, yai_process_t *out_process);
int yai_process_registry_list(yai_process_t *entries, size_t cap, size_t *out_len);

#ifdef __cplusplus
}
#endif

// include/yai/sdk/registry/registry_registry.h
#pragma once

#include "yai/sdk/registry/registry_cache.h"

#ifdef __cplusplus
extern "C" {
#endif

// Init builds indexes (id -> command, group -> list) on first use.
// Returns 0 on success.
int yai_law_registry_init(void);

// Registry pointers (cache is immutable; indexes live in process memory).
const yai_law_registry_t* yai_law_registry(void);

// Fast lookups (require init).
const yai_law_command_t* yai_law_cmd_by_id(const char* id);

// Group iteration (require init).
typedef struct yai_law_cmd_list {
  const yai_law_command_t** items;
  size_t len;
} yai_law_cmd_list_t;

yai_law_cmd_list_t yai_law_cmds_by_group(const char* group);

// Utility
int yai_law_command_has_output(const yai_law_command_t* c, const char* out);
int yai_law_command_has_side_effect(const yai_law_command_t* c, const char* eff);

#ifdef __cplusplus
}
#endif
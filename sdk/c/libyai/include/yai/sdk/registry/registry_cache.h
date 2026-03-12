/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#include <yai/sdk/registry/registry_types.h>

/*
 * Registry cache: stores/returns a loaded registry instance.
 * This header must NOT define any registry types. Types live in
 * law_registry_types.h (single source of truth).
 */

typedef struct yai_law_registry_cache {
    yai_law_registry_t registry;
    int loaded; /* 0/1 */
} yai_law_registry_cache_t;

/* Initialize cache (no load performed). */
void yai_law_registry_cache_init(yai_law_registry_cache_t *cache);

/* Clear cache contents. */
void yai_law_registry_cache_clear(yai_law_registry_cache_t *cache);

/* Load registry into cache if not loaded; returns 0 on success. */
int yai_law_registry_cache_load(yai_law_registry_cache_t *cache);

/* Get loaded registry (NULL if not loaded). */
const yai_law_registry_t *yai_law_registry_cache_get(const yai_law_registry_cache_t *cache);

void yai_law_registry_cache_free(yai_law_registry_cache_t *cache);

int yai_law_registry_cache_load_from_files(
    yai_law_registry_cache_t *cache,
    const char *commands_json_path,
    const char *artifacts_json_path);

#ifdef __cplusplus
}
#endif
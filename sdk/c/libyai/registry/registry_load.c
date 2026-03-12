/* SPDX-License-Identifier: Apache-2.0 */
// src/model/registry/registry_load.c

#include "yai/sdk/registry/registry_registry.h"
#include "yai/sdk/registry/registry_cache.h"
#include "yai/sdk/registry/registry_validate.h"

#include <stddef.h>
#include <stdio.h>

static yai_law_registry_cache_t g_cache;
static const yai_law_registry_t* g_reg = NULL;
static int g_inited = 0;
static int g_init_rc = 1;

int yai_law_registry_init(void)
{
    if (g_inited) return g_init_rc;
    g_inited = 1;

    yai_law_registry_cache_init(&g_cache);

    /* 1) Load registry into cache (generated/embedded or from file, depending on your impl) */
    if (yai_law_registry_cache_load(&g_cache) != 0) {
        g_reg = NULL;
        g_init_rc = 1;
        return g_init_rc;
    }

    const yai_law_registry_t* reg = yai_law_registry_cache_get(&g_cache);
    if (!reg) {
        g_reg = NULL;
        g_init_rc = 1;
        return g_init_rc;
    }

    /* 2) Validate (structural checks; no IO) */
    if (yai_law_registry_validate_all(reg) != 0) {
        g_reg = NULL;
        g_init_rc = 1;
        return g_init_rc;
    }

    /* 3) Publish */
    g_reg = reg;
    g_init_rc = 0;
    return g_init_rc;
}

const yai_law_registry_t* yai_law_registry(void)
{
    return g_reg;
}
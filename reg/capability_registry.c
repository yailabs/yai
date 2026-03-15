#include <stddef.h>
#include <string.h>

#include "yai/abi/errors.h"
#include "yai/kernel/registry.h"

#define YAI_CAPABILITY_REGISTRY_MAX 64

static struct yai_capability_registry_entry g_entries[YAI_CAPABILITY_REGISTRY_MAX];
static uint8_t g_used[YAI_CAPABILITY_REGISTRY_MAX];

int yai_capability_registry_set(const struct yai_capability_registry_entry* entry) {
    size_t i;

    if (entry == 0 || entry->key[0] == '\0') {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_CAPABILITY_REGISTRY_MAX; ++i) {
        if (g_used[i] && strncmp(g_entries[i].key, entry->key, YAI_REGISTRY_KEY_MAX) == 0) {
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    for (i = 0; i < YAI_CAPABILITY_REGISTRY_MAX; ++i) {
        if (!g_used[i]) {
            g_used[i] = 1;
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    return YAI_ERR_BUSY;
}

int yai_capability_registry_get(const char* key, struct yai_capability_registry_entry* out_entry) {
    size_t i;

    if (key == 0 || out_entry == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_CAPABILITY_REGISTRY_MAX; ++i) {
        if (g_used[i] && strncmp(g_entries[i].key, key, YAI_REGISTRY_KEY_MAX) == 0) {
            *out_entry = g_entries[i];
            return YAI_OK;
        }
    }

    return YAI_ERR_NOT_FOUND;
}

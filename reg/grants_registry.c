#include <stddef.h>

#include "yai/abi/errors.h"
#include "yai/kernel/registry.h"

#define YAI_GRANTS_REGISTRY_MAX 128

static yai_grants_registry_entry g_entries[YAI_GRANTS_REGISTRY_MAX];
static uint8_t g_used[YAI_GRANTS_REGISTRY_MAX];

int yai_grants_registry_set(const yai_grants_registry_entry* entry) {
    size_t i;

    if (entry == 0 || entry->grant_id == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_GRANTS_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].grant_id == entry->grant_id) {
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    for (i = 0; i < YAI_GRANTS_REGISTRY_MAX; ++i) {
        if (!g_used[i]) {
            g_used[i] = 1;
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    return YAI_ERR_BUSY;
}

int yai_grants_registry_get(yai_object_id_t grant_id, yai_grants_registry_entry* out_entry) {
    size_t i;

    if (grant_id == 0 || out_entry == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_GRANTS_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].grant_id == grant_id) {
            *out_entry = g_entries[i];
            return YAI_OK;
        }
    }

    return YAI_ERR_NOT_FOUND;
}

int yai_grants_registry_find_active(
    yai_object_id_t subject_handle,
    enum yai_kernel_capability_class capability_class,
    yai_object_id_t scope_handle,
    yai_grants_registry_entry* out_entry) {
    size_t i;

    if (subject_handle == 0 || out_entry == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_GRANTS_REGISTRY_MAX; ++i) {
        if (!g_used[i]) {
            continue;
        }

        if (g_entries[i].validity_state != YAI_KERNEL_GRANT_ACTIVE) {
            continue;
        }

        if (g_entries[i].subject_handle != subject_handle) {
            continue;
        }

        if (g_entries[i].capability_class != capability_class) {
            continue;
        }

        if (scope_handle != 0 && g_entries[i].scope_handle != scope_handle) {
            continue;
        }

        *out_entry = g_entries[i];
        return YAI_OK;
    }

    return YAI_ERR_NOT_FOUND;
}

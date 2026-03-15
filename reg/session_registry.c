#include <stddef.h>

#include "yai/abi/errors.h"
#include "yai/kernel/registry.h"

#define YAI_SESSION_REGISTRY_MAX 256

static yai_session_registry_entry g_entries[YAI_SESSION_REGISTRY_MAX];
static uint8_t g_used[YAI_SESSION_REGISTRY_MAX];

int yai_session_registry_upsert(const yai_session_registry_entry* entry) {
    size_t i;

    if (entry == 0 || entry->session_id == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_SESSION_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].session_id == entry->session_id) {
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    for (i = 0; i < YAI_SESSION_REGISTRY_MAX; ++i) {
        if (!g_used[i]) {
            g_used[i] = 1;
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    return YAI_ERR_BUSY;
}

int yai_session_registry_get(yai_object_id_t session_id, yai_session_registry_entry* out_entry) {
    size_t i;

    if (session_id == 0 || out_entry == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_SESSION_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].session_id == session_id) {
            *out_entry = g_entries[i];
            return YAI_OK;
        }
    }

    return YAI_ERR_NOT_FOUND;
}

#include <stddef.h>

#include "yai/abi/errors.h"
#include "yai/kernel/registry.h"

#define YAI_TRACE_REGISTRY_MAX 64

static struct yai_trace_registry_entry g_entries[YAI_TRACE_REGISTRY_MAX];
static uint8_t g_used[YAI_TRACE_REGISTRY_MAX];

int yai_trace_registry_set(const struct yai_trace_registry_entry* entry) {
    size_t i;

    if (entry == 0 || entry->channel_id == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_TRACE_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].channel_id == entry->channel_id) {
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    for (i = 0; i < YAI_TRACE_REGISTRY_MAX; ++i) {
        if (!g_used[i]) {
            g_used[i] = 1;
            g_entries[i] = *entry;
            return YAI_OK;
        }
    }

    return YAI_ERR_BUSY;
}

int yai_trace_registry_get(yai_object_id_t channel_id, struct yai_trace_registry_entry* out_entry) {
    size_t i;

    if (channel_id == 0 || out_entry == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_TRACE_REGISTRY_MAX; ++i) {
        if (g_used[i] && g_entries[i].channel_id == channel_id) {
            *out_entry = g_entries[i];
            return YAI_OK;
        }
    }

    return YAI_ERR_NOT_FOUND;
}

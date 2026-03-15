#include <stddef.h>
#include <string.h>

#include "yai/abi/errors.h"
#include "yai/kernel/registry.h"

#define YAI_KERNEL_REGISTRY_MAX 64

struct kernel_registry_entry {
    char key[YAI_REGISTRY_KEY_MAX];
    uint64_t value;
    int used;
};

static struct kernel_registry_entry g_entries[YAI_KERNEL_REGISTRY_MAX];

void yai_kernel_registry_bootstrap(struct yai_kernel_registry_roots* out_roots) {
    memset(g_entries, 0, sizeof(g_entries));

    if (out_roots == 0) {
        return;
    }

    out_roots->kernel_registry = YAI_REGISTRY_HANDLE_KERNEL;
    out_roots->container_registry = YAI_REGISTRY_HANDLE_CONTAINER;
    out_roots->session_registry = YAI_REGISTRY_HANDLE_SESSION;
    out_roots->capability_registry = YAI_REGISTRY_HANDLE_CAPABILITY;
    out_roots->containment_registry = YAI_REGISTRY_HANDLE_CONTAINMENT;
    out_roots->trace_registry = YAI_REGISTRY_HANDLE_TRACE;
    out_roots->grants_registry = YAI_REGISTRY_HANDLE_GRANTS;
}

int yai_kernel_registry_register(const char* key, uint64_t value) {
    size_t i;

    if (key == 0 || key[0] == '\0') {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_KERNEL_REGISTRY_MAX; ++i) {
        if (g_entries[i].used && strcmp(g_entries[i].key, key) == 0) {
            g_entries[i].value = value;
            return YAI_OK;
        }
    }

    for (i = 0; i < YAI_KERNEL_REGISTRY_MAX; ++i) {
        if (!g_entries[i].used) {
            g_entries[i].used = 1;
            strncpy(g_entries[i].key, key, sizeof(g_entries[i].key) - 1);
            g_entries[i].key[sizeof(g_entries[i].key) - 1] = '\0';
            g_entries[i].value = value;
            return YAI_OK;
        }
    }

    return YAI_ERR_BUSY;
}

int yai_kernel_registry_lookup(const char* key, uint64_t* out_value) {
    size_t i;

    if (key == 0 || out_value == 0) {
        return YAI_ERR_INVALID;
    }

    for (i = 0; i < YAI_KERNEL_REGISTRY_MAX; ++i) {
        if (g_entries[i].used && strcmp(g_entries[i].key, key) == 0) {
            *out_value = g_entries[i].value;
            return YAI_OK;
        }
    }

    return YAI_ERR_NOT_FOUND;
}

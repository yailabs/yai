#pragma once

#ifndef YAI_KERNEL_REGISTRY_H
#define YAI_KERNEL_REGISTRY_H

#include <stdint.h>

#include "grants.h"
#include "handles.h"
#include "objects.h"
#include <yai/security/containment.h>
#include "session.h"

#define YAI_REGISTRY_KEY_MAX 48

struct yai_kernel_registry_roots {
    yai_registry_root_id_t kernel_registry;
    yai_registry_root_id_t container_registry;
    yai_registry_root_id_t session_registry;
    yai_registry_root_id_t capability_registry;
    yai_registry_root_id_t containment_registry;
    yai_registry_root_id_t trace_registry;
    yai_registry_root_id_t grants_registry;
};

typedef struct yai_kernel_session yai_session_registry_entry;
typedef struct yai_security_containment_state yai_containment_registry_entry;
typedef struct yai_kernel_grant yai_grants_registry_entry;

struct yai_capability_registry_entry {
    char key[YAI_REGISTRY_KEY_MAX];
    uint64_t cap_mask;
    uint8_t enabled;
};

struct yai_trace_registry_entry {
    yai_object_id_t channel_id;
    uint64_t event_mask;
    uint64_t metric_mask;
    uint8_t enabled;
};

void yai_kernel_registry_bootstrap(struct yai_kernel_registry_roots* out_roots);

int yai_kernel_registry_register(const char* key, uint64_t value);
int yai_kernel_registry_lookup(const char* key, uint64_t* out_value);

int yai_session_registry_upsert(const yai_session_registry_entry* entry);
int yai_session_registry_get(yai_object_id_t session_id, yai_session_registry_entry* out_entry);

int yai_capability_registry_set(const struct yai_capability_registry_entry* entry);
int yai_capability_registry_get(const char* key, struct yai_capability_registry_entry* out_entry);

int yai_containment_registry_set(const yai_containment_registry_entry* entry);
int yai_containment_registry_get(yai_object_id_t container_id, yai_containment_registry_entry* out_entry);

int yai_trace_registry_set(const struct yai_trace_registry_entry* entry);
int yai_trace_registry_get(yai_object_id_t channel_id, struct yai_trace_registry_entry* out_entry);

int yai_grants_registry_set(const yai_grants_registry_entry* entry);
int yai_grants_registry_get(yai_object_id_t grant_id, yai_grants_registry_entry* out_entry);
int yai_grants_registry_find_active(
    yai_object_id_t subject_handle,
    enum yai_kernel_capability_class capability_class,
    yai_object_id_t scope_handle,
    yai_grants_registry_entry* out_entry);

#endif /* YAI_KERNEL_REGISTRY_H */

#include "yai/abi/errors.h"
#include "yai/kernel/capabilities.h"
#include "yai/kernel/state.h"

static struct yai_kernel_state g_state;

void yai_kernel_state_init(uint64_t boot_id, const struct yai_kernel_registry_roots* roots) {
    g_state.boot_id = boot_id;
    g_state.tick = 0;
    g_state.mode_flags = 0;

    g_state.lifecycle.current_state = YAI_KERNEL_STATE_BOOTING;
    g_state.lifecycle.previous_state = YAI_KERNEL_STATE_BOOTING;
    g_state.lifecycle.readiness_flags = 0;
    g_state.lifecycle.last_transition_reason = 0;
    g_state.lifecycle.entered_at = 0;
    g_state.lifecycle.flags = 0;

    g_state.capability.root_caps = yai_kernel_root_caps();
    g_state.capability.ready_subsystems = 0;

    if (roots != 0) {
        g_state.roots = *roots;
    }

    g_state.admitted_sessions = 0;
    g_state.active_containers = 0;
    g_state.audit_channels = 0;
}

int yai_kernel_state_set_lifecycle(
    enum yai_kernel_lifecycle_state_id next_state,
    uint32_t reason,
    uint64_t entered_at) {
    g_state.lifecycle.previous_state = g_state.lifecycle.current_state;
    g_state.lifecycle.current_state = next_state;
    g_state.lifecycle.last_transition_reason = reason;
    g_state.lifecycle.entered_at = entered_at;
    return YAI_OK;
}

int yai_kernel_state_set_readiness(uint64_t readiness_flag, int ready) {
    if (ready) {
        g_state.lifecycle.readiness_flags |= readiness_flag;
    } else {
        g_state.lifecycle.readiness_flags &= ~readiness_flag;
    }
    return YAI_OK;
}

int yai_kernel_state_has_readiness(uint64_t readiness_mask) {
    if ((g_state.lifecycle.readiness_flags & readiness_mask) == readiness_mask) {
        return YAI_OK;
    }
    return YAI_ERR_DENIED;
}

int yai_kernel_state_set_subsystem_ready(uint64_t subsystem_flag, int ready) {
    if (ready) {
        g_state.capability.ready_subsystems |= subsystem_flag;
    } else {
        g_state.capability.ready_subsystems &= ~subsystem_flag;
    }
    return YAI_OK;
}

void yai_kernel_state_inc_sessions(void) {
    g_state.admitted_sessions += 1;
}

void yai_kernel_state_inc_containers(void) {
    g_state.active_containers += 1;
}

void yai_kernel_state_inc_audit_channels(void) {
    g_state.audit_channels += 1;
}

const struct yai_kernel_state* yai_kernel_state_get(void) {
    return &g_state;
}

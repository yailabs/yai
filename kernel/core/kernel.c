#include "yai/kernel/capabilities.h"
#include "yai/kernel/kernel.h"
#include "yai/kernel/lifecycle.h"
#include "yai/kernel/state.h"

uint64_t yai_kernel_root_caps(void) {
    return YAI_CAP_PROCESS |
           YAI_CAP_MEMORY |
           YAI_CAP_IPC |
           YAI_CAP_NET |
           YAI_CAP_NAMESPACE;
}

void yai_kernel_bootstrap(uint64_t boot_id) {
    yai_kernel_boot_begin(boot_id, 1);

    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_PROC, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_SCHED, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_MM, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_FS, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_IPC, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_SECURITY, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_CONTAINER, 1);
    yai_kernel_state_set_subsystem_ready(YAI_SUBSYS_AUDIT, 1);

    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_PROC, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_SCHED, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_MM, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_FS, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_IPC, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_SECURITY, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_CONTAINER_PRIMITIVE, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_AUDIT, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_SESSION_ADMISSION, 1);
    yai_kernel_mark_subsystem_ready(YAI_KERNEL_READY_POLICY_GRANTS, 1);

    yai_kernel_enter_ready(0, 2);
}

void yai_kernel_tick(void) {
    const struct yai_kernel_state* state = yai_kernel_state_get();
    (void)state;
}

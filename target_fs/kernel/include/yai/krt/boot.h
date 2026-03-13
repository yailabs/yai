#pragma once

#ifndef YAI_KERNEL_BOOT_H
#define YAI_KERNEL_BOOT_H

#include <stdint.h>

#include <yai/container/lifecycle.h>

#define YAI_KERNEL_BOOT_HANDOFF_MAGIC 0x594149424F4F5431ULL /* "YAIBOOT1" */
#define YAI_KERNEL_BOOT_HANDOFF_VERSION 1u

enum yai_kernel_boot_mode {
    YAI_KERNEL_BOOT_MODE_NORMAL = 0,
    YAI_KERNEL_BOOT_MODE_RECOVERY = 1,
    YAI_KERNEL_BOOT_MODE_DIAGNOSTIC = 2
};

struct yai_kernel_boot_handoff {
    uint64_t magic;
    uint32_t version;
    enum yai_kernel_boot_mode mode;
    uint64_t boot_id;
    uint64_t issued_at;
    uint64_t flags;
    char boot_slot[16];
    char initrd_label[32];
};

struct yai_kernel_boot_report {
    uint64_t boot_id;
    int handoff_ok;
    int preboot_ok;
    int layout_ok;
    int lifecycle_ok;
    enum yai_kernel_lifecycle_state_id final_state;
    uint64_t readiness_flags;
    char reason[96];
};

int yai_kernel_boot_validate_handoff(const struct yai_kernel_boot_handoff* handoff);
int yai_kernel_preboot_checks(const struct yai_kernel_boot_handoff* handoff, struct yai_kernel_boot_report* out_report);
int yai_kernel_boot_execute(const struct yai_kernel_boot_handoff* handoff, struct yai_kernel_boot_report* out_report);

#endif /* YAI_KERNEL_BOOT_H */

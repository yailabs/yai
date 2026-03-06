/* SPDX-License-Identifier: Apache-2.0 */
/* Kernel public state and transition declarations. */
#ifndef YAI_KERNEL_H
#define YAI_KERNEL_H

#include "yai_vault.h"
#include <stdint.h>
#include <stdbool.h>
#include <time.h>

/* Canonical runtime states used by kernel execution flow. */
typedef enum {
    YAI_STATE_CREATED = 0,
    YAI_STATE_PROVISIONED,
    YAI_STATE_CONTEXT_READY,
    YAI_STATE_EXECUTING,
    YAI_STATE_VALIDATING,
    YAI_STATE_COMMITTED,
    YAI_STATE_ABORTED,
    YAI_STATE_TERMINATED_BY_RUNTIME,
    YAI_STATE_TERMINATED
} yai_run_state_t;

typedef struct {
    uint32_t capability_id;
    uint32_t run_id;
    time_t expires_at;
    bool revoked;
    uint32_t scope_mask;
} yai_grant_t;

void yai_scan_workspace(const char *path, int depth);
/* Kernel FSM transition (vault->status via yai_state_t). */
int yai_kernel_transition(yai_vault_t *vault, yai_state_t to_state);

#endif

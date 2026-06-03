/*
 * YAI - Process carrier
 *
 * Implements:
 *   POSIX process observation and test-owned signal control.
 *
 * This file owns:
 *   Process carrier v0 safety posture and carrier.v1 receipt mapping.
 *
 * This file does not own:
 *   Arbitrary process killing, shell execution, workflow spawning or queues.
 */

#define _POSIX_C_SOURCE 200809L

#include "yai/effect/process_carrier.h"

#include "yai/effect/carrier_contract.h"

#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

void yai_copy_string(char *dst, size_t dst_size, const char *src);

static void process_receipt_init(yai_process_carrier_receipt_t *receipt) {
    if (receipt == 0) return;
    memset(receipt, 0, sizeof(*receipt));
    (void)yai_carrier_receipt_init(&receipt->carrier_receipt);
    receipt->carrier_receipt.carrier_family = YAI_CARRIER_FAMILY_PROCESS;
    yai_copy_string(receipt->carrier_receipt.carrier_name, sizeof(receipt->carrier_receipt.carrier_name), "process");
    yai_copy_string(receipt->carrier_receipt.contract_version, sizeof(receipt->carrier_receipt.contract_version), YAI_CARRIER_CONTRACT_V1);
    receipt->carrier_receipt.guarantee_mode = YAI_RECEIPT_GUARANTEE_INTERPOSED;
    receipt->carrier_receipt.pre_state_observed = 1;
    receipt->carrier_receipt.evidence_captured = 1;
    receipt->operation[0] = '\0';
    receipt->signal = YAI_PROCESS_SIGNAL_UNKNOWN;
    receipt->pre_state = YAI_PROCESS_STATE_UNKNOWN;
    receipt->post_state = YAI_PROCESS_STATE_UNKNOWN;
    receipt->carrier_attempted = 0;
    yai_copy_string(receipt->reason, sizeof(receipt->reason), "");
}

static yai_process_state_t observe_pid(pid_t pid) {
    if (pid <= 0) {
        return YAI_PROCESS_STATE_NOT_FOUND;
    }
    if (kill(pid, 0) == 0) {
        return YAI_PROCESS_STATE_RUNNING;
    }
    if (errno == ESRCH) {
        return YAI_PROCESS_STATE_NOT_FOUND;
    }
    if (errno == EPERM) {
        return YAI_PROCESS_STATE_PERMISSION_DENIED;
    }
    return YAI_PROCESS_STATE_UNKNOWN;
}

const char *yai_process_owner_scope_to_string(yai_process_owner_scope_t scope) {
    switch (scope) {
    case YAI_PROCESS_OWNER_TEST_OWNED:
        return "test_owned";
    case YAI_PROCESS_OWNER_YAI_OWNED:
        return "yai_owned";
    case YAI_PROCESS_OWNER_EXTERNAL_OBSERVED:
        return "external_observed";
    case YAI_PROCESS_OWNER_UNKNOWN:
        return "unknown";
    default:
        return "unknown";
    }
}

yai_process_owner_scope_t yai_process_owner_scope_from_string(const char *value) {
    if (value == 0) return YAI_PROCESS_OWNER_UNKNOWN;
    if (strcmp(value, "test_owned") == 0) return YAI_PROCESS_OWNER_TEST_OWNED;
    if (strcmp(value, "yai_owned") == 0) return YAI_PROCESS_OWNER_YAI_OWNED;
    if (strcmp(value, "external_observed") == 0) return YAI_PROCESS_OWNER_EXTERNAL_OBSERVED;
    return YAI_PROCESS_OWNER_UNKNOWN;
}

const char *yai_process_carrier_contract_version(void) {
    return YAI_CARRIER_CONTRACT_V1;
}

yai_status_t yai_process_target_init(yai_process_target_t *target,
                                     pid_t pid,
                                     yai_process_owner_scope_t owner_scope,
                                     int spawned_by_yai,
                                     const char *command_preview) {
    if (target == 0) {
        return YAI_ERR_INVALID;
    }
    memset(target, 0, sizeof(*target));
    target->pid = pid;
    target->owner_scope = owner_scope;
    target->spawned_by_yai = spawned_by_yai ? 1 : 0;
    target->observed_state = YAI_PROCESS_STATE_UNKNOWN;
    (void)snprintf(target->process_ref, sizeof(target->process_ref), "process:%ld", (long)pid);
    yai_copy_string(target->command_preview, sizeof(target->command_preview), command_preview ? command_preview : "");
    return YAI_OK;
}

yai_status_t yai_process_carrier_observe(yai_process_target_t *target,
                                         yai_process_carrier_receipt_t *receipt) {
    if (target == 0 || receipt == 0) {
        return YAI_ERR_INVALID;
    }
    process_receipt_init(receipt);
    yai_copy_string(receipt->operation, sizeof(receipt->operation), "process.observe");
    receipt->pid = target->pid;
    receipt->pre_state = observe_pid(target->pid);
    receipt->post_state = receipt->pre_state;
    target->observed_state = receipt->pre_state;
    receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_OBSERVED;
    receipt->carrier_receipt.post_state_observed = 1;
    receipt->carrier_receipt.receipt_validated = 1;
    receipt->carrier_receipt.residue_recorded = 1;
    yai_copy_string(receipt->reason, sizeof(receipt->reason), "process_observed");
    return YAI_OK;
}

yai_status_t yai_process_carrier_signal(yai_process_target_t *target,
                                        yai_process_signal_t signal,
                                        int dry_run,
                                        yai_process_carrier_receipt_t *receipt) {
    int posix_signal = yai_process_signal_to_posix(signal);
    int wait_status = 0;
    pid_t waited = -1;

    if (target == 0 || receipt == 0) {
        return YAI_ERR_INVALID;
    }
    process_receipt_init(receipt);
    yai_copy_string(receipt->operation, sizeof(receipt->operation), "process.signal");
    receipt->pid = target->pid;
    receipt->signal = signal;
    receipt->pre_state = observe_pid(target->pid);
    target->observed_state = receipt->pre_state;

    if (dry_run) {
        receipt->post_state = receipt->pre_state;
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_NOT_ATTEMPTED;
        receipt->carrier_receipt.post_state_observed = 1;
        receipt->carrier_receipt.receipt_validated = 1;
        receipt->carrier_receipt.residue_recorded = 1;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "dry_run");
        return YAI_OK;
    }

    if (!(target->owner_scope == YAI_PROCESS_OWNER_TEST_OWNED && target->spawned_by_yai)) {
        receipt->post_state = receipt->pre_state;
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_BLOCKED;
        receipt->carrier_receipt.post_state_observed = 1;
        receipt->carrier_receipt.receipt_validated = 1;
        receipt->carrier_receipt.residue_recorded = 1;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "unsafe_process_target");
        return YAI_OK;
    }

    if (!(signal == YAI_PROCESS_SIGNAL_TERM || signal == YAI_PROCESS_SIGNAL_KILL) || posix_signal == 0) {
        receipt->post_state = receipt->pre_state;
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_BLOCKED;
        receipt->carrier_receipt.post_state_observed = 1;
        receipt->carrier_receipt.receipt_validated = 1;
        receipt->carrier_receipt.residue_recorded = 1;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "unsupported_signal");
        return YAI_OK;
    }

    if (receipt->pre_state != YAI_PROCESS_STATE_RUNNING) {
        receipt->post_state = receipt->pre_state;
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_FAILED;
        receipt->carrier_receipt.post_state_observed = 1;
        receipt->carrier_receipt.receipt_validated = 1;
        receipt->carrier_receipt.residue_recorded = 1;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "process_not_running");
        return YAI_OK;
    }

    /* CORE.ENFORCE.1: a real dispatcher must pass yai_dispatch_admit (lease +
     * decision) before reaching this signal path. The carrier independently
     * enforces the test-owned/spawned-by-yai safety posture above as defense in
     * depth; no execution is attempted outside it. */
    receipt->carrier_attempted = 1;
    if (kill(target->pid, posix_signal) != 0) {
        receipt->post_state = observe_pid(target->pid);
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_FAILED;
        receipt->carrier_receipt.post_state_observed = 1;
        receipt->carrier_receipt.receipt_validated = 1;
        receipt->carrier_receipt.residue_recorded = 1;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "signal_failed");
        return YAI_OK;
    }

    waited = waitpid(target->pid, &wait_status, 0);
    if (waited == target->pid) {
        if (WIFSIGNALED(wait_status)) {
            receipt->post_state = YAI_PROCESS_STATE_SIGNALED;
        } else if (WIFEXITED(wait_status)) {
            receipt->post_state = YAI_PROCESS_STATE_EXITED;
        } else if (WIFSTOPPED(wait_status)) {
            receipt->post_state = YAI_PROCESS_STATE_STOPPED;
        } else {
            receipt->post_state = YAI_PROCESS_STATE_UNKNOWN;
        }
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_EXECUTED;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "signal_sent");
    } else {
        receipt->post_state = observe_pid(target->pid);
        receipt->carrier_receipt.outcome = YAI_CARRIER_OUTCOME_FAILED;
        yai_copy_string(receipt->reason, sizeof(receipt->reason), "waitpid_failed");
    }
    receipt->carrier_receipt.post_state_observed = 1;
    receipt->carrier_receipt.receipt_validated = 1;
    receipt->carrier_receipt.residue_recorded = 1;
    target->observed_state = receipt->post_state;
    return YAI_OK;
}

#include "yai/effect/filesystem_carrier.h"

#include "yai/effect/carrier_contract.h"
#include "yai/effect/effect_hash.h"

#include <stdio.h>
#include <string.h>

void yai_copy_string(char *dst, size_t dst_size, const char *src);

static int yai_path_contains_parent_segment(const char *path) {
    const char *cursor = path;

    if (path == 0) {
        return 1;
    }

    if (strcmp(path, "..") == 0) {
        return 1;
    }

    while ((cursor = strstr(cursor, "..")) != 0) {
        int before_ok = cursor == path || cursor[-1] == '/';
        int after_ok = cursor[2] == '\0' || cursor[2] == '/';
        if (before_ok && after_ok) {
            return 1;
        }
        cursor += 2;
    }
    return 0;
}

int yai_filesystem_path_is_inside_sandbox(const char *sandbox_root,
                                          const char *path) {
    size_t sandbox_len = 0;

    if (sandbox_root == 0 || sandbox_root[0] == '\0' ||
        path == 0 || path[0] == '\0' ||
        yai_path_contains_parent_segment(sandbox_root) ||
        yai_path_contains_parent_segment(path)) {
        return 0;
    }

    sandbox_len = strlen(sandbox_root);
    if (strncmp(path, sandbox_root, sandbox_len) != 0) {
        return 0;
    }

    return path[sandbox_len] == '\0' || path[sandbox_len] == '/';
}

const char *yai_filesystem_carrier_contract_version(void) {
    return YAI_CARRIER_CONTRACT_V1;
}

yai_status_t yai_filesystem_carrier_receipt_from_effect_receipt(
    const yai_effect_receipt_t *effect_receipt,
    yai_carrier_receipt_t *out
) {
    return yai_carrier_receipt_from_effect_receipt(effect_receipt, out);
}

static yai_status_t yai_filesystem_receipt_init(const char *receipt_id,
                                                const yai_op_attempt_t *attempt,
                                                const yai_control_decision_t *decision,
                                                yai_effect_kind_t effect_kind,
                                                const char *path,
                                                yai_effect_receipt_t *receipt) {
    if (receipt == 0 || receipt_id == 0 || receipt_id[0] == '\0' ||
        attempt == 0 || decision == 0 || path == 0 || path[0] == '\0') {
        return YAI_ERR_INVALID;
    }

    yai_id_set(&receipt->receipt_id, receipt_id);
    receipt->case_ref = attempt->case_ref;
    receipt->attempt_id = attempt->attempt_id;
    receipt->decision_id = decision->decision_id;
    receipt->target_subject_ref = attempt->target_subject_ref;
    receipt->carrier_kind = YAI_CARRIER_FILESYSTEM;
    receipt->effect_kind = effect_kind;
    yai_copy_string(receipt->path_or_locator, sizeof(receipt->path_or_locator), path);
    receipt->bytes_affected = 0;
    yai_copy_string(receipt->before_hash, sizeof(receipt->before_hash), "");
    yai_copy_string(receipt->after_hash, sizeof(receipt->after_hash), "");
    yai_copy_string(receipt->summary, sizeof(receipt->summary), "");
    receipt->status = YAI_RECEIPT_UNKNOWN;
    return YAI_OK;
}

yai_status_t yai_filesystem_carrier_read(const char *receipt_id,
                                         const yai_op_attempt_t *attempt,
                                         const yai_control_decision_t *decision,
                                         const char *sandbox_root,
                                         const char *path,
                                         yai_effect_receipt_t *receipt) {
    yai_status_t status = YAI_OK;
    size_t byte_count = 0;

    status = yai_filesystem_receipt_init(receipt_id,
                                         attempt,
                                         decision,
                                         YAI_EFFECT_FS_READ,
                                         path,
                                         receipt);
    if (status != YAI_OK) {
        return status;
    }

    if (!yai_filesystem_path_is_inside_sandbox(sandbox_root, path)) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.read outside sandbox blocked");
        return YAI_OK;
    }

    status = yai_effect_hash_file(path, receipt->after_hash, sizeof(receipt->after_hash), &byte_count);
    if (status != YAI_OK) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.read failed");
        return YAI_OK;
    }

    receipt->status = YAI_RECEIPT_OBSERVED;
    receipt->bytes_affected = byte_count;
    yai_copy_string(receipt->before_hash, sizeof(receipt->before_hash), receipt->after_hash);
    yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.read observed");
    return YAI_OK;
}

yai_status_t yai_filesystem_carrier_write(const char *receipt_id,
                                          const yai_op_attempt_t *attempt,
                                          const yai_control_decision_t *decision,
                                          const char *sandbox_root,
                                          const char *path,
                                          const char *content,
                                          yai_effect_receipt_t *receipt) {
    FILE *file = 0;
    yai_status_t status = YAI_OK;
    size_t before_bytes = 0;
    size_t written = 0;
    size_t content_len = 0;

    status = yai_filesystem_receipt_init(receipt_id,
                                         attempt,
                                         decision,
                                         YAI_EFFECT_FS_WRITE,
                                         path,
                                         receipt);
    if (status != YAI_OK) {
        return status;
    }

    if (content == 0) {
        content = "";
    }
    content_len = strlen(content);

    if (!yai_filesystem_path_is_inside_sandbox(sandbox_root, path)) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write outside sandbox blocked");
        return YAI_OK;
    }

    /* CORE.ENFORCE.1: a real dispatcher must pass yai_dispatch_admit (lease +
     * decision) before invoking this carrier. The carrier independently
     * re-checks the decision outcome here as defense in depth; a non-allow
     * decision never executes. */
    if (!(decision->outcome == YAI_DECISION_ALLOW ||
          decision->outcome == YAI_DECISION_ALLOW_WITH_CONSTRAINTS)) {
        receipt->status = YAI_RECEIPT_BLOCKED;
        yai_copy_string(receipt->before_hash, sizeof(receipt->before_hash), "not_written");
        yai_copy_string(receipt->after_hash, sizeof(receipt->after_hash), "not_written");
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write blocked by control decision");
        return YAI_OK;
    }

    status = yai_effect_hash_file(path, receipt->before_hash, sizeof(receipt->before_hash), &before_bytes);
    if (status == YAI_ERR_NOT_FOUND) {
        yai_copy_string(receipt->before_hash, sizeof(receipt->before_hash), "none");
    } else if (status != YAI_OK) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write before hash failed");
        return YAI_OK;
    }

    file = fopen(path, "wb");
    if (file == 0) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write open failed");
        return YAI_OK;
    }
    written = fwrite(content, 1, content_len, file);
    if (written != content_len) {
        (void)fclose(file);
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write incomplete");
        return YAI_OK;
    }
    if (fclose(file) != 0) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write close failed");
        return YAI_OK;
    }

    status = yai_effect_hash_file(path, receipt->after_hash, sizeof(receipt->after_hash), &receipt->bytes_affected);
    if (status != YAI_OK) {
        receipt->status = YAI_RECEIPT_FAILED;
        yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write after hash failed");
        return YAI_OK;
    }

    receipt->status = YAI_RECEIPT_EXECUTED;
    yai_copy_string(receipt->summary, sizeof(receipt->summary), "fs.write executed");
    return YAI_OK;
}

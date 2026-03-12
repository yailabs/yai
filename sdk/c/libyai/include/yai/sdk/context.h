/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Canonical container-context API.
 * Runtime payload interoperability may still carry workspace-keyed fields.
 */

/* Returns 0 and writes current container id when present.
 * Returns 1 when no current container binding is set.
 * Returns -1 on I/O/validation errors. */
int yai_sdk_context_get_current_container(char *out_container_id, size_t out_cap);

/* Sets current container binding.
 * Returns 0 on success, -1 on validation/I/O errors. */
int yai_sdk_context_set_current_container(const char *container_id);

/* Switches current container binding.
 * Returns 0 on success, -1 on validation/I/O errors. */
int yai_sdk_context_switch_container(const char *container_id);

/* Unsets current container binding.
 * Returns 0 on success (also when already absent), -1 on I/O errors. */
int yai_sdk_context_unset_container(void);

/* Clears current container binding (alias for unset).
 * Returns 0 on success (also when already absent), -1 on I/O errors. */
int yai_sdk_context_clear_current_container(void);

/* Resolve effective container id by precedence:
 * 1) explicit container id when set
 * 2) current container context
 * Returns:
 * 0 -> resolved
 * 1 -> no container context available
 * -1 -> errors */
int yai_sdk_context_resolve_container(
    const char *explicit_container_id,
    char *out_container_id,
    size_t out_cap);

typedef struct yai_sdk_container_info {
    char ws_id[64];
    int exists;
    int valid;
    char state[32];
    char root_path[512];
} yai_sdk_container_info_t;

/* Runtime-backed container descriptor using runtime container status command. */
int yai_sdk_container_describe(const char *container_id, yai_sdk_container_info_t *out);

/* Validates current binding against runtime state.
 * Returns 0 when current binding exists and runtime confirms a valid container. */
int yai_sdk_context_validate_current_container(yai_sdk_container_info_t *out);

#ifdef __cplusplus
}
#endif

/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

/*
 * Law help printer (registry-driven).
 *
 * - global: overview + groups
 * - group: list commands in group
 * - any: tries id/name/group match (in that order)
 *
 * Returns an exit code (0 success; >0 means "not found" or error).
 */

int yai_law_help_print_global(void);
int yai_law_help_print_group(const char *group);
int yai_law_help_print_any(const char *tok);

#ifdef __cplusplus
}
#endif
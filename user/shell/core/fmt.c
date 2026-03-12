// SPDX-License-Identifier: Apache-2.0
// src/util/fmt.c

#include "yai/shell/fmt.h"

#include <stdio.h>
#include <string.h>

/*
 * Formatting layer (shell output only).
 *
 * No parsing.
 * No transformation.
 * Server output is considered authoritative.
 */

static void print_line_ensure_nl(const char *s)
{
    if (!s) return;
    fputs(s, stdout);
    size_t n = strlen(s);
    if (n == 0 || s[n - 1] != '\n') fputc('\n', stdout);
}

void yai_print_response(const char *payload, int json_mode)
{
    if (!payload) return;

    if (json_mode) {
        /* Raw mode: exact server payload */
        print_line_ensure_nl(payload);
        return;
    }

    /* Human mode: deterministic prefix */
    fputs("[yai] ", stdout);
    print_line_ensure_nl(payload);
}

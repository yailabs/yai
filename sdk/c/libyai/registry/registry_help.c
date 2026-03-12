/* SPDX-License-Identifier: Apache-2.0 */
// src/model/registry/registry_help.c

#define _POSIX_C_SOURCE 200809L

#include "yai/sdk/registry/registry_help.h"
#include "yai/sdk/registry/registry_cache.h"
#include "yai/sdk/registry/registry_validate.h"

#include <stdio.h>
#include <string.h>

static yai_law_registry_cache_t g_cache;

static const yai_law_registry_t* load_registry_or_null(void)
{
    /* lazy init */
    static int inited = 0;
    if (!inited) {
        yai_law_registry_cache_init(&g_cache);
        inited = 1;
    }

    if (!g_cache.loaded) {
        if (yai_law_registry_cache_load(&g_cache) != 0)
            return NULL;

        const yai_law_registry_t *r = yai_law_registry_cache_get(&g_cache);
        if (!r) return NULL;

        /* validate once on load (hard fail) */
        int vrc = yai_law_registry_validate_all(r);
        if (vrc != 0) {
            fprintf(stderr, "ERR: registry validation failed (rc=%d)\n", vrc);
            return NULL;
        }
    }

    return yai_law_registry_cache_get(&g_cache);
}

static void print_command_brief(const yai_law_command_t *c)
{
    if (!c) return;
    printf("  %-18s %-18s %s\n",
           c->group ? c->group : "-",
           c->name ? c->name : "-",
           c->summary ? c->summary : "");
}

static void print_command_detail(const yai_law_command_t *c)
{
    if (!c) return;

    printf("%s\n", c->name ? c->name : "(unnamed)");
    printf("  id:      %s\n", c->id ? c->id : "-");
    printf("  group:   %s\n", c->group ? c->group : "-");
    printf("  summary: %s\n", c->summary ? c->summary : "-");

    if (c->args_len > 0 && c->args) {
        printf("\nArgs:\n");
        for (size_t i = 0; i < c->args_len; i++) {
            const yai_law_arg_t *a = &c->args[i];
            printf("  - %s", a->name ? a->name : "(noname)");
            if (a->flag && a->flag[0]) printf(" (%s)", a->flag);
            if (a->pos > 0) printf(" [pos=%d]", (int)a->pos);
            printf(" : %s", a->type ? a->type : "-");
            if (a->required) printf(" (required)");
            if (a->default_s && a->default_s[0]) printf(" [default=\"%s\"]", a->default_s);
            if (a->default_b_set) printf(" [default=%s]", a->default_b ? "true" : "false");
            if (a->default_i_set) printf(" [default=%lld]", (long long)a->default_i);

            if (a->values && a->values_len > 0) {
                printf(" {");
                for (size_t j = 0; j < a->values_len; j++) {
                    printf("%s%s", (j ? "," : ""), a->values[j] ? a->values[j] : "");
                }
                printf("}");
            }

            printf("\n");
        }
    }

    if (c->outputs && c->outputs_len > 0) {
        printf("\nOutputs:\n");
        for (size_t i = 0; i < c->outputs_len; i++) {
            printf("  - %s\n", c->outputs[i] ? c->outputs[i] : "");
        }
    }

    if (c->side_effects && c->side_effects_len > 0) {
        printf("\nSide effects:\n");
        for (size_t i = 0; i < c->side_effects_len; i++) {
            printf("  - %s\n", c->side_effects[i] ? c->side_effects[i] : "");
        }
    }

    if (c->law_invariants && c->law_invariants_len > 0) {
        printf("\nInvariants:\n");
        for (size_t i = 0; i < c->law_invariants_len; i++) {
            printf("  - %s\n", c->law_invariants[i] ? c->law_invariants[i] : "");
        }
    }

    if (c->law_boundaries && c->law_boundaries_len > 0) {
        printf("\nBoundaries:\n");
        for (size_t i = 0; i < c->law_boundaries_len; i++) {
            printf("  - %s\n", c->law_boundaries[i] ? c->law_boundaries[i] : "");
        }
    }

    if (c->uses_primitives && c->uses_primitives_len > 0) {
        printf("\nUses primitives:\n");
        for (size_t i = 0; i < c->uses_primitives_len; i++) {
            printf("  - %s\n", c->uses_primitives[i] ? c->uses_primitives[i] : "");
        }
    }
}

static int group_exists(const yai_law_registry_t *r, const char *group)
{
    if (!r || !group) return 0;
    for (size_t i = 0; i < r->commands_len; i++) {
        const yai_law_command_t *c = &r->commands[i];
        if (c->group && strcmp(c->group, group) == 0)
            return 1;
    }
    return 0;
}

static const yai_law_command_t* find_by_id(const yai_law_registry_t *r, const char *id)
{
    if (!r || !id) return NULL;
    for (size_t i = 0; i < r->commands_len; i++) {
        const yai_law_command_t *c = &r->commands[i];
        if (c->id && strcmp(c->id, id) == 0) return c;
    }
    return NULL;
}

static const yai_law_command_t* find_by_name(const yai_law_registry_t *r, const char *name)
{
    if (!r || !name) return NULL;
    for (size_t i = 0; i < r->commands_len; i++) {
        const yai_law_command_t *c = &r->commands[i];
        if (c->name && strcmp(c->name, name) == 0) return c;
    }
    return NULL;
}

int yai_law_help_print_global(void)
{
    const yai_law_registry_t *r = load_registry_or_null();
    if (!r) {
        fprintf(stderr, "ERR: registry not available\n");
        return 2;
    }

    printf("YAI Law Registry\n");
    if (r->version) printf("  version: %s\n", r->version);
    if (r->binary)  printf("  binary:  %s\n", r->binary);
    printf("\nCommands (%zu):\n", r->commands_len);

    /* print all commands in brief */
    for (size_t i = 0; i < r->commands_len; i++) {
        print_command_brief(&r->commands[i]);
    }

    printf("\nHint:\n");
    printf("  yai law help <entrypoint>\n");
    printf("  yai law help <entrypoint> <topic>\n");
    printf("  yai law help <command>\n");
    printf("  yai law help <command_id>\n");

    return 0;
}

int yai_law_help_print_group(const char *group)
{
    if (!group || !group[0]) return 2;

    const yai_law_registry_t *r = load_registry_or_null();
    if (!r) {
        fprintf(stderr, "ERR: registry not available\n");
        return 2;
    }

    if (!group_exists(r, group)) {
        fprintf(stderr, "ERR: unknown group: %s\n", group);
        return 3;
    }

    printf("Group: %s\n\n", group);
    for (size_t i = 0; i < r->commands_len; i++) {
        const yai_law_command_t *c = &r->commands[i];
        if (c->group && strcmp(c->group, group) == 0) {
            print_command_brief(c);
        }
    }

    return 0;
}

int yai_law_help_print_any(const char *tok)
{
    if (!tok || !tok[0]) return 2;

    const yai_law_registry_t *r = load_registry_or_null();
    if (!r) {
        fprintf(stderr, "ERR: registry not available\n");
        return 2;
    }

    /* priority: id -> name -> group */
    const yai_law_command_t *c = find_by_id(r, tok);
    if (!c) c = find_by_name(r, tok);

    if (c) {
        print_command_detail(c);
        return 0;
    }

    if (group_exists(r, tok)) {
        return yai_law_help_print_group(tok);
    }

    fprintf(stderr, "ERR: unknown help topic: %s\n", tok);
    return 3;
}

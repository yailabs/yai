// SPDX-License-Identifier: Apache-2.0
/*
 * YAI init/main.c
 *
 * Canonical YAI bring-up spine:
 * boot -> arch -> memory -> ipc -> security -> graph -> policy
 *      -> container -> runtime -> userspace
 */

#include <stdio.h>
#include <stdlib.h>

static int yai_boot_stage(void);
static int yai_arch_stage(void);
static int yai_memory_stage(void);
static int yai_ipc_stage(void);
static int yai_security_stage(void);
static int yai_graph_stage(void);
static int yai_policy_stage(void);
static int yai_container_stage(void);
static int yai_runtime_stage(void);
static int yai_userspace_stage(void);

static int yai_fail(const char *stage)
{
    fprintf(stderr, "[yai:init] stage failed: %s\n", stage);
    return 1;
}

static int yai_boot_stage(void)
{
    return 0;
}

static int yai_arch_stage(void)
{
    return 0;
}

static int yai_memory_stage(void)
{
    return 0;
}

static int yai_ipc_stage(void)
{
    return 0;
}

static int yai_security_stage(void)
{
    return 0;
}

static int yai_graph_stage(void)
{
    return 0;
}

static int yai_policy_stage(void)
{
    return 0;
}

static int yai_container_stage(void)
{
    return 0;
}

static int yai_runtime_stage(void)
{
    return 0;
}

static int yai_userspace_stage(void)
{
    return 0;
}

int start_kernel(void)
{
    if (yai_boot_stage() != 0) return yai_fail("boot");
    if (yai_arch_stage() != 0) return yai_fail("arch");
    if (yai_memory_stage() != 0) return yai_fail("memory");
    if (yai_ipc_stage() != 0) return yai_fail("ipc");
    if (yai_security_stage() != 0) return yai_fail("security");
    if (yai_graph_stage() != 0) return yai_fail("graph");
    if (yai_policy_stage() != 0) return yai_fail("policy");
    if (yai_container_stage() != 0) return yai_fail("container");
    if (yai_runtime_stage() != 0) return yai_fail("runtime");
    if (yai_userspace_stage() != 0) return yai_fail("userspace");
    return 0;
}

int main(void)
{
    return start_kernel();
}

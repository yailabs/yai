// SPDX-License-Identifier: Apache-2.0

#include "version.h"
#include <yai/log.h>

static struct yai_boot_identity g_yai_boot_identity = {
    .sysname = "YAI",
    .nodename = "yai-node",
    .release = YAI_UTS_RELEASE,
    .version = YAI_UTS_VERSION,
    .machine = YAI_UTS_MACHINE,
    .domainname = "local",
};

const char yai_proc_banner[] =
    "%s version %s (%s@%s) (%s) %s\n";

const char yai_banner[] =
    "YAI version " YAI_UTS_RELEASE " ("
    YAI_COMPILE_BY "@"
    YAI_COMPILE_HOST ") ("
    YAI_COMPILER ") "
    YAI_UTS_VERSION "\n";

void yai_init_boot_identity(void)
{
    g_yai_boot_identity.release = YAI_UTS_RELEASE;
    g_yai_boot_identity.version = YAI_UTS_VERSION;
    g_yai_boot_identity.machine = YAI_UTS_MACHINE;
}

int yai_set_early_hostname(const char *hostname)
{
    size_t i;

    if (!hostname || !hostname[0]) {
        return -1;
    }

    for (i = 0; hostname[i] != '\0'; ++i) {
        if (i >= sizeof(g_yai_boot_identity.nodename) - 1) {
            yai_log_warn("early hostname truncated to %zu bytes",
                         sizeof(g_yai_boot_identity.nodename) - 1);
            break;
        }
        g_yai_boot_identity.nodename[i] = hostname[i];
    }

    g_yai_boot_identity.nodename[i] = '\0';
    return 0;
}

const struct yai_boot_identity *yai_get_boot_identity(void)
{
    return &g_yai_boot_identity;
}

const char *yai_get_banner(void)
{
    return yai_banner;
}
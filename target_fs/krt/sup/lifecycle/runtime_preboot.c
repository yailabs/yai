#define _POSIX_C_SOURCE 200809L
#include <yai/sup/lifecycle.h>
#include <yai/api/runtime.h>
#include <yai/data/binding.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <errno.h>
#include <limits.h>

static int mkdir_safe(const char *path)
{
    if (mkdir(path, 0700) == 0)
        return 0;

    if (errno == EEXIST)
        return 0;

    perror("[PREBOOT] mkdir failed");
    return -1;
}

int yai_ensure_runtime_layout(const char *ws_id)
{
    const char *home = getenv("HOME");
    if (!home)
        return -1;

    char path[PATH_MAX];

    /* ~/.yai */
    snprintf(path, sizeof(path), "%s/.yai", home);
    if (mkdir_safe(path) != 0)
        return -2;

    /* ~/.yai/run */
    snprintf(path, sizeof(path), "%s/.yai/run", home);
    if (mkdir_safe(path) != 0)
        return -3;

    /* workspace */
    if (ws_id && ws_id[0]) {
        snprintf(path, sizeof(path), "%s/.yai/run/%s", home, ws_id);
        if (mkdir_safe(path) != 0)
            return -4;
    }

    {
        char bind_err[96];
        if (yai_data_store_binding_init(bind_err, sizeof(bind_err)) != 0) {
            fprintf(stderr, "[PREBOOT] data store binding init failed: %s\n", bind_err);
            return -5;
        }
    }

    if (ws_id && ws_id[0]) {
        char bind_err[96];
        if (yai_data_store_binding_attach_scope(ws_id, bind_err, sizeof(bind_err)) != 0) {
            fprintf(stderr, "[PREBOOT] workspace data binding failed: %s\n", bind_err);
            return -6;
        }
    }

    return 0;
}

int yai_run_preboot_checks(void)
{
    if (getuid() == 0)
        printf("[PREBOOT] Warning: Running as root is not recommended\n");

    return 0;
}

#define _XOPEN_SOURCE 700
#define _POSIX_C_SOURCE 200809L

#include <yai/supervisor/lifecycle.h>
#include <yai/kernel/vault.h>

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#define SYSTEM_WS "system"
#define SHM_PREFIX "/yai_vault_"

static int yai_format_system_shm_name(char *out, size_t out_sz)
{
    if (!out || out_sz == 0) {
        return -1;
    }

    if (snprintf(out, out_sz, "%s%s", SHM_PREFIX, SYSTEM_WS) >= (int)out_sz) {
        return -2;
    }

    return 0;
}

int yai_init_system_shm(void)
{
    char name[64];
    int fd = -1;
    yai_vault_t *vault = NULL;
    int rc = 0;

    rc = yai_format_system_shm_name(name, sizeof(name));
    if (rc != 0) {
        fprintf(stderr, "[YAI-RUNTIME] failed to format system shm name (rc=%d)\n", rc);
        return -1;
    }

    (void)shm_unlink(name);

    fd = shm_open(name, O_CREAT | O_RDWR, 0666);
    if (fd < 0) {
        perror("[YAI-RUNTIME] shm_open failed");
        return -2;
    }

    if (ftruncate(fd, (off_t)sizeof(yai_vault_t)) != 0) {
        perror("[YAI-RUNTIME] ftruncate failed");
        close(fd);
        return -3;
    }

    vault = mmap(NULL,
                 sizeof(yai_vault_t),
                 PROT_READ | PROT_WRITE,
                 MAP_SHARED,
                 fd,
                 0);
    if (vault == MAP_FAILED) {
        perror("[YAI-RUNTIME] mmap failed");
        close(fd);
        return -4;
    }

    memset(vault, 0, sizeof(yai_vault_t));
    strncpy(vault->workspace_id, SYSTEM_WS, MAX_WS_ID - 1);
    vault->workspace_id[MAX_WS_ID - 1] = '\0';

    if (munmap(vault, sizeof(yai_vault_t)) != 0) {
        perror("[YAI-RUNTIME] munmap failed");
        close(fd);
        return -5;
    }

    if (close(fd) != 0) {
        perror("[YAI-RUNTIME] close failed");
        return -6;
    }

    printf("[YAI-RUNTIME] system shared memory initialized (%s)\n", name);
    return 0;
}

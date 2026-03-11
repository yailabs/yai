#include <yai/orchestration/engine_bridge.h>

#include <sys/mman.h>
#include <fcntl.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <time.h>
#include <errno.h>
#include <sys/stat.h>

static yai_exec_vault_t* _vault = NULL;
static int _shm_fd = -1;

static yai_exec_vault_t* attach_shm(const char* shm_path) {
    int fd = shm_open(shm_path, O_RDWR, 0666);
    if (fd == -1) return NULL;

    // best-effort size sanity: require at least Vault size
    struct stat st;
    if (fstat(fd, &st) != 0) {
        close(fd);
        return NULL;
    }
    if ((size_t)st.st_size < sizeof(yai_exec_vault_t)) {
        close(fd);
        return NULL;
    }

    yai_exec_vault_t* v = (yai_exec_vault_t*)mmap(NULL, sizeof(yai_exec_vault_t), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    close(fd);
    if (v == MAP_FAILED) return NULL;
    return v;
}

int yai_bridge_init(const char* ws_id) {
    if (!ws_id || !ws_id[0]) return -1;

    char shm_path[128];
    snprintf(shm_path, sizeof(shm_path), "%s%s", SHM_VAULT_PREFIX, ws_id);

    _shm_fd = shm_open(shm_path, O_RDWR, 0666);
    if (_shm_fd == -1) return -2;

    _vault = (yai_exec_vault_t*)mmap(NULL, sizeof(yai_exec_vault_t), PROT_READ | PROT_WRITE, MAP_SHARED, _shm_fd, 0);
    if (_vault == MAP_FAILED) {
        close(_shm_fd);
        _shm_fd = -1;
        _vault = NULL;
        return -3;
    }

    // DO NOT mutate kernel-owned state here.
    return 0;
}

yai_exec_vault_t* yai_bridge_attach(const char* ws_id, const char* channel) {
    if (!ws_id || !ws_id[0]) return NULL;

    char shm_path[128];
    char base_path[128];

    snprintf(base_path, sizeof(base_path), "%s%s", SHM_VAULT_PREFIX, ws_id);

    if (channel && channel[0] != '\0') {
        snprintf(shm_path, sizeof(shm_path), "%s%s_%s", SHM_VAULT_PREFIX, ws_id, channel);
    } else {
        snprintf(shm_path, sizeof(shm_path), "%s%s", SHM_VAULT_PREFIX, ws_id);
    }

    yai_exec_vault_t* v = attach_shm(shm_path);
    if (!v && strcmp(shm_path, base_path) != 0) {
        v = attach_shm(base_path);
    }
    if (!v) return NULL;

    // Phase-2: CORE attach selects default vault pointer
    if (!channel || strcmp(channel, "CORE") == 0) {
        _vault = v;
    }
    return v;
}

yai_exec_vault_t* yai_get_vault(void) { return _vault; }

bool yai_consume_energy(uint32_t amount) {
    if (!_vault) return false;
    if (_vault->authority_lock) return false;

    if (_vault->energy_consumed + amount > _vault->energy_quota) return false;
    _vault->energy_consumed += amount;
    return true;
}

void yai_bridge_detach(void) {
    if (_vault) {
        munmap(_vault, sizeof(yai_exec_vault_t));
        _vault = NULL;
    }
    if (_shm_fd != -1) {
        close(_shm_fd);
        _shm_fd = -1;
    }
}

void yai_audit_log_transition(const char* action, uint32_t prev_state, uint32_t new_state) {
    FILE* f = fopen("engine_runtime.trace", "a");
    if (!f) return;

    yai_exec_vault_t* v = yai_get_vault();
    fprintf(f, "%ld,%s,%u,%u,%u\n",
            time(NULL),
            action ? action : "unknown",
            prev_state,
            new_state,
            v ? v->energy_consumed : 0);

    fclose(f);
    fprintf(stderr, "[TLA-AUDIT] %s: %u -> %u\n", action ? action : "unknown", prev_state, new_state);
}

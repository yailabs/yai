#ifndef YAI_SESSION_H
#define YAI_SESSION_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <protocol.h>
#include <transport.h>

#define MAX_SESSIONS    32
#define MAX_WS_ID_LEN   64
#define MAX_PATH_LEN    512

typedef uint32_t yai_cap_mask_t;

#define YAI_CAP_NONE            0u
#define YAI_CAP_RPC_PING        (1u << 0)
#define YAI_CAP_RPC_HANDSHAKE   (1u << 1)
#define YAI_CAP_RPC_STATUS      (1u << 2)

typedef enum {
    YAI_WS_CREATED = 0,
    YAI_WS_ACTIVE,
    YAI_WS_ATTACHED,
    YAI_WS_SUSPENDED,
    YAI_WS_DESTROYED,
    YAI_WS_ERROR
} yai_ws_state_t;

typedef struct {
    char ws_id[MAX_WS_ID_LEN];
    char run_dir[MAX_PATH_LEN];
    char root_path[MAX_PATH_LEN];
    char control_sock[MAX_PATH_LEN];
    char lock_file[MAX_PATH_LEN];
    char pid_file[MAX_PATH_LEN];
    long created_at;
    long updated_at;
    yai_ws_state_t state;
} yai_workspace_t;

typedef struct {
    char ws_id[MAX_WS_ID_LEN];
    int exists;
    char state[24];
    char root_path[MAX_PATH_LEN];
    char layout[32];
    long created_at;
    long updated_at;
} yai_workspace_runtime_info_t;

typedef struct {
    uint32_t session_id;
    uint32_t run_id;
    yai_workspace_t ws;
    yai_cap_mask_t caps;
    uint32_t owner_pid;
} yai_session_t;

extern yai_session_t g_session_registry[MAX_SESSIONS];

bool yai_ws_validate_id(const char* ws_id);
bool yai_ws_build_paths(yai_workspace_t* ws, const char* ws_id);
bool yai_session_acquire(yai_session_t** out, const char* ws_id);
void yai_session_release(yai_session_t* s);

/* ✅ NEW BINARY SAFE DISPATCH */
void yai_session_dispatch(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload
);

#endif

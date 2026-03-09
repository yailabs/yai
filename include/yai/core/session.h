#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <yai/protocol/rpc_runtime.h>
#include <yai/protocol/transport_contract.h>

#define MAX_SESSIONS 32
#define MAX_WS_ID_LEN 64
#define MAX_PATH_LEN 512

typedef uint32_t yai_cap_mask_t;

#define YAI_CAP_NONE 0u
#define YAI_CAP_RPC_PING (1u << 0)
#define YAI_CAP_RPC_HANDSHAKE (1u << 1)
#define YAI_CAP_RPC_STATUS (1u << 2)

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
  char ingress_sock[MAX_PATH_LEN];
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
  char workspace_store_root[MAX_PATH_LEN];
  char runtime_state_root[MAX_PATH_LEN];
  char metadata_root[MAX_PATH_LEN];
  char root_anchor_mode[32];
  char layout[32];
  char workspace_alias[64];
  char session_binding[64];
  int runtime_attached;
  int control_plane_attached;
  char declared_control_family[96];
  char declared_specialization[96];
  char declared_context_source[24];
  char inferred_family[96];
  char inferred_specialization[96];
  double inferred_confidence;
  char effective_stack_ref[192];
  char effective_overlays_ref[192];
  char last_effect_summary[192];
  char last_authority_summary[192];
  char last_evidence_summary[192];
  char last_resolution_summary[192];
  char isolation_mode[24];
  int debug_mode;
  char last_resolution_trace_ref[192];
  char shell_cwd[MAX_PATH_LEN];
  char shell_path_relation[24];
  long created_at;
  long activated_at;
  long last_attached_at;
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

bool yai_ws_validate_id(const char *ws_id);
bool yai_ws_build_paths(yai_workspace_t *ws, const char *ws_id);
bool yai_session_acquire(yai_session_t **out, const char *ws_id);
void yai_session_release(yai_session_t *s);

void yai_session_dispatch(int client_fd, const yai_rpc_envelope_t *env, const char *payload);

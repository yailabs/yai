#pragma once

#include <stdbool.h>
#include <ctype.h>
#include <stddef.h>
#include <stdint.h>

#define YAI_WS_ID_MAX 35u
#define YAI_WS_ALIAS_MAX 63u
#define YAI_WS_PATH_MAX 511u
#define YAI_WS_CTX_NAME_MAX 95u
#define YAI_WS_REF_MAX 191u

static inline int yai_ws_id_is_valid(const char *ws_id) {
  if (!ws_id) {
    return 0;
  }

  size_t n = 0;
  while (n <= YAI_WS_ID_MAX && ws_id[n] != '\0') {
    n++;
  }
  if (n == 0 || n > YAI_WS_ID_MAX) {
    return 0;
  }

  for (size_t i = 0; i < n; i++) {
    unsigned char c = (unsigned char)ws_id[i];
    if (!(isalnum(c) || c == '-' || c == '_')) {
      return 0;
    }
  }
  return 1;
}

typedef enum {
  YAI_WORKSPACE_STATE_CREATED = 0,
  YAI_WORKSPACE_STATE_ACTIVE,
  YAI_WORKSPACE_STATE_ATTACHED,
  YAI_WORKSPACE_STATE_SUSPENDED,
  YAI_WORKSPACE_STATE_DESTROYED,
  YAI_WORKSPACE_STATE_ERROR,
} yai_workspace_state_t;

typedef enum {
  YAI_WORKSPACE_CONTEXT_SOURCE_UNSET = 0,
  YAI_WORKSPACE_CONTEXT_SOURCE_DECLARED,
  YAI_WORKSPACE_CONTEXT_SOURCE_INFERRED,
  YAI_WORKSPACE_CONTEXT_SOURCE_RESTORED,
} yai_workspace_context_source_t;

typedef enum {
  YAI_WORKSPACE_ISOLATION_PROCESS = 0,
  YAI_WORKSPACE_ISOLATION_WORKSPACE_FS,
  YAI_WORKSPACE_ISOLATION_STRICT,
} yai_workspace_isolation_mode_t;

typedef struct {
  char workspace_id[YAI_WS_ID_MAX + 1u];
  char workspace_alias[YAI_WS_ALIAS_MAX + 1u];
  char workspace_root[YAI_WS_PATH_MAX + 1u];
} yai_workspace_identity_t;

typedef struct {
  yai_workspace_state_t workspace_state;
  int64_t created_at;
  int64_t activated_at;
  int64_t last_attached_at;
  int64_t last_updated_at;
} yai_workspace_lifecycle_t;

typedef struct {
  char session_binding[YAI_WS_ID_MAX + 1u];
  int runtime_attached;
  char runtime_endpoint[YAI_WS_REF_MAX + 1u];
  int control_plane_attached;
} yai_workspace_runtime_binding_t;

typedef struct {
  char declared_control_family[YAI_WS_CTX_NAME_MAX + 1u];
  char declared_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  char declared_profile[YAI_WS_CTX_NAME_MAX + 1u];
  yai_workspace_context_source_t declared_context_source;
} yai_workspace_declared_context_t;

typedef struct {
  char last_inferred_family[YAI_WS_CTX_NAME_MAX + 1u];
  char last_inferred_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  double last_inference_confidence;
} yai_workspace_inferred_context_t;

typedef struct {
  char effective_stack_ref[YAI_WS_REF_MAX + 1u];
  char effective_overlays_ref[YAI_WS_REF_MAX + 1u];
  char last_effect_summary[YAI_WS_REF_MAX + 1u];
  char last_authority_summary[YAI_WS_REF_MAX + 1u];
  char last_evidence_summary[YAI_WS_REF_MAX + 1u];
} yai_workspace_effective_state_t;

typedef struct {
  char event_id[YAI_WS_REF_MAX + 1u];
  char flow_stage[YAI_WS_CTX_NAME_MAX + 1u];
  char declared_scenario_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  char business_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  char enforcement_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  int external_effect_boundary;
} yai_workspace_event_surface_t;

typedef struct {
  char binding_state[YAI_WS_CTX_NAME_MAX + 1u];
  char attached_governance_objects[YAI_WS_REF_MAX + 1u];
  char active_effective_stack[YAI_WS_REF_MAX + 1u];
  char last_event_ref[YAI_WS_REF_MAX + 1u];
  char last_flow_stage[YAI_WS_CTX_NAME_MAX + 1u];
  char last_business_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  char last_enforcement_specialization[YAI_WS_CTX_NAME_MAX + 1u];
  char last_effect[YAI_WS_REF_MAX + 1u];
  char last_authority[YAI_WS_REF_MAX + 1u];
  char last_evidence[YAI_WS_REF_MAX + 1u];
  char last_trace_ref[YAI_WS_REF_MAX + 1u];
  char review_state[YAI_WS_CTX_NAME_MAX + 1u];
  char operational_summary[YAI_WS_REF_MAX + 1u];
} yai_workspace_operational_state_t;

typedef struct {
  yai_workspace_isolation_mode_t isolation_mode;
  int debug_mode;
  char last_resolution_trace_ref[YAI_WS_REF_MAX + 1u];
} yai_workspace_runtime_flags_t;

typedef struct {
  yai_workspace_identity_t identity;
  yai_workspace_lifecycle_t lifecycle;
  yai_workspace_runtime_binding_t binding;
  yai_workspace_declared_context_t declared;
  yai_workspace_inferred_context_t inferred;
  yai_workspace_effective_state_t effective;
  yai_workspace_event_surface_t event_surface;
  yai_workspace_operational_state_t operational;
  yai_workspace_runtime_flags_t runtime;
} yai_workspace_manifest_v1_t;

typedef struct {
  yai_workspace_identity_t identity;
  yai_workspace_state_t workspace_state;
  yai_workspace_runtime_binding_t binding;
  yai_workspace_declared_context_t declared;
  yai_workspace_inferred_context_t inferred;
  yai_workspace_effective_state_t effective;
  yai_workspace_event_surface_t event_surface;
  yai_workspace_operational_state_t operational;
  yai_workspace_runtime_flags_t runtime;
  char last_resolution_summary[YAI_WS_REF_MAX + 1u];
} yai_workspace_inspect_v1_t;

const char *yai_workspace_state_name(yai_workspace_state_t state);
const char *yai_workspace_context_source_name(yai_workspace_context_source_t source);
const char *yai_workspace_isolation_mode_name(yai_workspace_isolation_mode_t mode);
int yai_workspace_registry_is_online(void);
void yai_workspace_manifest_defaults(yai_workspace_manifest_v1_t *manifest,
                                     const char *workspace_id,
                                     const char *workspace_root);
void yai_workspace_inspect_from_manifest(const yai_workspace_manifest_v1_t *manifest,
                                         yai_workspace_inspect_v1_t *inspect);
int yai_workspace_bind_runtime_capabilities(const char *workspace_id,
                                            char *err,
                                            size_t err_cap);
int yai_workspace_recover_runtime_capabilities(const char *workspace_id,
                                               char *err,
                                               size_t err_cap);

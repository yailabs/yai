#include "internal.h"

#include <stdio.h>
#include <string.h>

const char *yai_workspace_state_name(yai_workspace_state_t state) {
  switch (state) {
    case YAI_WORKSPACE_STATE_CREATED:
      return "created";
    case YAI_WORKSPACE_STATE_ACTIVE:
      return "active";
    case YAI_WORKSPACE_STATE_ATTACHED:
      return "attached";
    case YAI_WORKSPACE_STATE_SUSPENDED:
      return "suspended";
    case YAI_WORKSPACE_STATE_DESTROYED:
      return "destroyed";
    case YAI_WORKSPACE_STATE_ERROR:
      return "error";
    default:
      return "unknown";
  }
}

const char *yai_workspace_context_source_name(yai_workspace_context_source_t source) {
  switch (source) {
    case YAI_WORKSPACE_CONTEXT_SOURCE_UNSET:
      return "unset";
    case YAI_WORKSPACE_CONTEXT_SOURCE_DECLARED:
      return "declared";
    case YAI_WORKSPACE_CONTEXT_SOURCE_INFERRED:
      return "inferred";
    case YAI_WORKSPACE_CONTEXT_SOURCE_RESTORED:
      return "restored";
    default:
      return "unknown";
  }
}

const char *yai_workspace_isolation_mode_name(yai_workspace_isolation_mode_t mode) {
  switch (mode) {
    case YAI_WORKSPACE_ISOLATION_PROCESS:
      return "process";
    case YAI_WORKSPACE_ISOLATION_WORKSPACE_FS:
      return "workspace_fs";
    case YAI_WORKSPACE_ISOLATION_STRICT:
      return "strict";
    default:
      return "unknown";
  }
}

void yai_workspace_manifest_defaults(yai_workspace_manifest_v1_t *manifest,
                                     const char *workspace_id,
                                     const char *workspace_root) {
  if (!manifest) {
    return;
  }

  memset(manifest, 0, sizeof(*manifest));
  if (workspace_id) {
    snprintf(manifest->identity.workspace_id,
             sizeof(manifest->identity.workspace_id),
             "%s",
             workspace_id);
    snprintf(manifest->identity.workspace_alias,
             sizeof(manifest->identity.workspace_alias),
             "%s",
             workspace_id);
  }
  if (workspace_root) {
    snprintf(manifest->identity.workspace_root,
             sizeof(manifest->identity.workspace_root),
             "%s",
             workspace_root);
  }

  manifest->lifecycle.workspace_state = YAI_WORKSPACE_STATE_CREATED;
  manifest->declared.declared_context_source = YAI_WORKSPACE_CONTEXT_SOURCE_UNSET;
  manifest->runtime.isolation_mode = YAI_WORKSPACE_ISOLATION_PROCESS;
}

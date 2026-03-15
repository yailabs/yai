#pragma once

#define YAI_DAEMON_BINDING_KIND_OBSERVATIONAL "observational"
#define YAI_DAEMON_BINDING_KIND_MEDIABLE "mediable"

#define YAI_EDGE_SCOPE_NONE "none"
#define YAI_EDGE_SCOPE_WORKSPACE "workspace"

#define YAI_EDGE_MEDIATION_MODE_NONE "none"
#define YAI_EDGE_MEDIATION_MODE_HOLD_ESCALATE "hold_escalate"
#define YAI_EDGE_MEDIATION_MODE_ALLOW_BLOCK_HOLD "allow_block_hold"

const char *yai_edge_binding_kind_normalize(const char *raw);
const char *yai_edge_scope_normalize(const char *raw, const char *fallback);
const char *yai_edge_mediation_mode_normalize(const char *raw, const char *fallback);
int yai_edge_binding_is_mediable(const char *binding_kind);


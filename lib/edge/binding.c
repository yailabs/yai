#include <string.h>

#include <yai/edge/binding.h>

const char *yai_edge_binding_kind_normalize(const char *raw)
{
  if (!raw || !raw[0])
  {
    return YAI_EDGE_BINDING_KIND_OBSERVATIONAL;
  }
  if (strcmp(raw, YAI_EDGE_BINDING_KIND_MEDIABLE) == 0)
  {
    return YAI_EDGE_BINDING_KIND_MEDIABLE;
  }
  return YAI_EDGE_BINDING_KIND_OBSERVATIONAL;
}

const char *yai_edge_scope_normalize(const char *raw, const char *fallback)
{
  if (raw && raw[0])
  {
    return raw;
  }
  if (fallback && fallback[0])
  {
    return fallback;
  }
  return YAI_EDGE_SCOPE_NONE;
}

const char *yai_edge_mediation_mode_normalize(const char *raw, const char *fallback)
{
  const char *v = raw && raw[0] ? raw : fallback;
  if (!v || !v[0])
  {
    return YAI_EDGE_MEDIATION_MODE_NONE;
  }
  if (strcmp(v, YAI_EDGE_MEDIATION_MODE_ALLOW_BLOCK_HOLD) == 0)
  {
    return YAI_EDGE_MEDIATION_MODE_ALLOW_BLOCK_HOLD;
  }
  if (strcmp(v, YAI_EDGE_MEDIATION_MODE_HOLD_ESCALATE) == 0)
  {
    return YAI_EDGE_MEDIATION_MODE_HOLD_ESCALATE;
  }
  if (strcmp(v, YAI_EDGE_MEDIATION_MODE_NONE) == 0)
  {
    return YAI_EDGE_MEDIATION_MODE_NONE;
  }
  return YAI_EDGE_MEDIATION_MODE_NONE;
}

int yai_edge_binding_is_mediable(const char *binding_kind)
{
  return binding_kind && strcmp(binding_kind, YAI_EDGE_BINDING_KIND_MEDIABLE) == 0;
}


#include "internal.h"

#include <string.h>

#include <yai/core/lifecycle.h>
#include <yai/data/binding.h>

const char *yai_exec_runtime_state_name(yai_exec_runtime_state_t state) {
  switch (state) {
    case YAI_EXEC_OFFLINE:
      return "offline";
    case YAI_EXEC_READY:
      return "ready";
    case YAI_EXEC_BUSY:
      return "busy";
    case YAI_EXEC_ERROR:
      return "error";
    default:
      return "unknown";
  }
}

int yai_exec_runtime_probe(void) {
  const yai_runtime_capability_state_t *caps = yai_runtime_capabilities_state();

  if (yai_runtime_capabilities_is_ready() == 0) {
    return (int)YAI_EXEC_OFFLINE;
  }
  if (yai_data_store_binding_is_ready() == 0) {
    return (int)YAI_EXEC_ERROR;
  }
  if (!caps || !caps->workspace_id[0] || strcmp(caps->workspace_id, "system") == 0) {
    return (int)YAI_EXEC_BUSY;
  }
  return (int)YAI_EXEC_READY;
}

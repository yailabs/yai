#include "yai/container/bindings.h"
#include "yai/container/registry.h"

int yai_container_bindings_set(const char *container_id,
                               const yai_container_bindings_t *bindings) {
  yai_container_record_t record;

  if (!container_id || !bindings || container_id[0] == '\0') {
    return -1;
  }

  if (yai_container_registry_get_record(container_id, &record) != 0) {
    return -1;
  }

  record.state.daemon_bindings = bindings->daemon_binding_handle;
  record.state.network_bindings = bindings->network_binding_handle;

  return yai_container_registry_update(&record);
}

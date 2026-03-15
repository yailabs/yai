#include <yai/con/lifecycle.h>

int yai_container_lifecycle_transition_allowed(yai_container_lifecycle_state_t from,
                                               yai_container_lifecycle_state_t to) {
  switch (from) {
    case YAI_CONTAINER_LIFECYCLE_CREATED:
      return (to == YAI_CONTAINER_LIFECYCLE_INITIALIZED || to == YAI_CONTAINER_LIFECYCLE_DESTROYED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_INITIALIZED:
      return (to == YAI_CONTAINER_LIFECYCLE_OPEN || to == YAI_CONTAINER_LIFECYCLE_DEGRADED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_OPEN:
      return (to == YAI_CONTAINER_LIFECYCLE_ACTIVE ||
              to == YAI_CONTAINER_LIFECYCLE_DEGRADED ||
              to == YAI_CONTAINER_LIFECYCLE_SEALED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_ACTIVE:
      return (to == YAI_CONTAINER_LIFECYCLE_DEGRADED || to == YAI_CONTAINER_LIFECYCLE_SEALED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_DEGRADED:
      return (to == YAI_CONTAINER_LIFECYCLE_RECOVERY || to == YAI_CONTAINER_LIFECYCLE_SEALED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_RECOVERY:
      return (to == YAI_CONTAINER_LIFECYCLE_OPEN ||
              to == YAI_CONTAINER_LIFECYCLE_ACTIVE ||
              to == YAI_CONTAINER_LIFECYCLE_DEGRADED ||
              to == YAI_CONTAINER_LIFECYCLE_SEALED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_SEALED:
      return (to == YAI_CONTAINER_LIFECYCLE_DESTROYED || to == YAI_CONTAINER_LIFECYCLE_ARCHIVED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_DESTROYED:
      return (to == YAI_CONTAINER_LIFECYCLE_ARCHIVED) ? 1 : 0;
    case YAI_CONTAINER_LIFECYCLE_ARCHIVED:
    default:
      return 0;
  }
}

const char *yai_container_lifecycle_name(yai_container_lifecycle_state_t value) {
  switch (value) {
    case YAI_CONTAINER_LIFECYCLE_CREATED: return "created";
    case YAI_CONTAINER_LIFECYCLE_INITIALIZED: return "initialized";
    case YAI_CONTAINER_LIFECYCLE_OPEN: return "open";
    case YAI_CONTAINER_LIFECYCLE_ACTIVE: return "active";
    case YAI_CONTAINER_LIFECYCLE_DEGRADED: return "degraded";
    case YAI_CONTAINER_LIFECYCLE_SEALED: return "sealed";
    case YAI_CONTAINER_LIFECYCLE_RECOVERY: return "recovery";
    case YAI_CONTAINER_LIFECYCLE_DESTROYED: return "destroyed";
    case YAI_CONTAINER_LIFECYCLE_ARCHIVED: return "archived";
    default: return "unknown";
  }
}

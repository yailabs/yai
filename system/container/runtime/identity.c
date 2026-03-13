#include "yai/container/identity.h"

const char *yai_container_class_name(yai_container_class_t value) {
  switch (value) {
    case YAI_CONTAINER_CLASS_INTERACTIVE: return "interactive";
    case YAI_CONTAINER_CLASS_MANAGED: return "managed";
    case YAI_CONTAINER_CLASS_SERVICE: return "service";
    case YAI_CONTAINER_CLASS_SYSTEM: return "system";
    case YAI_CONTAINER_CLASS_RECOVERY: return "recovery";
    default: return "unknown";
  }
}

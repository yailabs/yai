#include <yai/mm/memory_class.h>

const char *yai_memory_class_name(yai_memory_class_t value) {
  switch (value) {
    case YAI_MEMORY_CLASS_RUNTIME: return "runtime";
    case YAI_MEMORY_CLASS_SHARED: return "shared";
    case YAI_MEMORY_CLASS_TRANSIENT: return "transient";
    case YAI_MEMORY_CLASS_UNSPECIFIED:
    default: return "unspecified";
  }
}

#pragma once

typedef enum {
  YAI_MEMORY_CLASS_UNSPECIFIED = 0,
  YAI_MEMORY_CLASS_RUNTIME,
  YAI_MEMORY_CLASS_SHARED,
  YAI_MEMORY_CLASS_TRANSIENT,
} yai_memory_class_t;

const char *yai_memory_class_name(yai_memory_class_t value);

// include/yai/sdk/registry/registry_registry_types.h
#pragma once

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_law_arg {
  const char* name;     // "ws"
  const char* flag;     // "--ws" (optional)
  int32_t     pos;      // 1..N (optional, 0 if not positional)
  const char* type;     // "string" | "bool" | "int" | "enum" | ...
  const char** values;  // enum values (optional)
  size_t      values_len;
  int         required; // 0/1

  // Defaults: support string/bool/int (extend as needed)
  const char* default_s;    // optional string default
  int         default_b_set;
  int         default_b;
  int         default_i_set;
  int64_t     default_i;
} yai_law_arg_t;

typedef struct yai_law_artifact_io {
  const char* role;       // "bundle_manifest"
  const char* schema_ref; // "model/schema/....schema.json" (optional, can be NULL)
  const char* path_hint;  // "bundle/**" (optional)
} yai_law_artifact_io_t;

typedef struct yai_law_command {
  const char* id;       // "yai.verify.verify"
  const char* name;     // "verify"
  const char* group;    // "verify"
  const char* summary;  // short

  // Taxonomy metadata (law-driven help/navigation)
  const char* surface;    // user|tool|internal
  const char* entrypoint; // taxonomy entrypoint token
  const char* topic;      // taxonomy topic token
  const char* op;         // e.g. status, trace, make
  const char* domain;     // container|runtime|governance|...
  const char* layer;      // taxonomy layer token
  const char* stability;  // stable|beta|experimental
  const char* canonical_path; // e.g. "gov decision status"
  int         help_order; // stable ordering hint for help renderers
  int         hidden;     // 0/1

  // NEW: alias tokens for "<group> <name>" resolution and UX shortcuts
  // Example: aliases ["runtime", "control-runtime"] etc. (implementation decides meaning)
  const char** aliases;   // optional
  size_t       aliases_len;

  // NEW: deprecation metadata
  int          deprecated;    // 0/1
  const char*  replaced_by;   // replacement command id (optional, required if deprecated=1)

  // NEW: lifecycle markers (optional strings; keep them opaque)
  // Examples: "0.1.0", "2026-02-28", etc.
  const char* since; // optional
  const char* until; // optional

  const yai_law_arg_t* args;
  size_t args_len;

  const char** outputs;       // ["text","json"] optional
  size_t outputs_len;

  const char** side_effects;  // optional
  size_t side_effects_len;

  const char** law_hooks;     // optional
  size_t law_hooks_len;

  const char** law_invariants; // optional
  size_t law_invariants_len;

  const char** law_boundaries; // optional
  size_t law_boundaries_len;

  const char** uses_primitives; // ["S-001","T-015",...]
  size_t uses_primitives_len;

  const yai_law_artifact_io_t* emits_artifacts;
  size_t emits_artifacts_len;

  const yai_law_artifact_io_t* consumes_artifacts;
  size_t consumes_artifacts_len;
} yai_law_command_t;

// Optional grouping layer for help UX (help --sets, curated lists)
typedef struct yai_law_command_set_item {
  const char* command; // canonical id "yai.<group>.<name>"
  const char* label;   // optional display label
} yai_law_command_set_item_t;

typedef struct yai_law_command_set {
  const char* id;          // "core", "ops", "governance", etc.
  const char* title;       // human name
  const char* description; // optional
  const yai_law_command_set_item_t* items;
  size_t items_len;
} yai_law_command_set_t;

typedef struct yai_law_artifact_role {
  const char* role;
  const char* schema_ref;
  const char* description;
} yai_law_artifact_role_t;

typedef struct yai_law_registry {
  const char* version; // commands.json "version" string (optional)
  const char* binary;  // "yai" (optional)

  // NEW: curated help sets (optional)
  const yai_law_command_set_t* command_sets;
  size_t command_sets_len;

  const yai_law_command_t* commands;
  size_t commands_len;

  const yai_law_artifact_role_t* artifacts;
  size_t artifacts_len;
} yai_law_registry_t;

#ifdef __cplusplus
}
#endif

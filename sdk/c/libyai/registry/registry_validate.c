// src/model/registry/registry_validate.c
#include "yai/sdk/registry/registry_validate.h"
#include "yai/sdk/registry/registry_cache.h"

#include <string.h>

static const yai_law_artifact_role_t* find_role(const yai_law_registry_t* r, const char* role) {
  if (!r || !role) return NULL;
  for (size_t i = 0; i < r->artifacts_len; i++) {
    const yai_law_artifact_role_t* a = &r->artifacts[i];
    if (a->role && strcmp(a->role, role) == 0) return a;
  }
  return NULL;
}

static int validate_args(const yai_law_command_t* c) {
  // minimal sanity: name/type required; if pos present must be >=1; flag optional
  for (size_t i = 0; i < c->args_len; i++) {
    const yai_law_arg_t* a = &c->args[i];
    if (!a->name || !a->type) return 10;
    if (a->pos < 0) return 11;
    if (a->values_len > 0 && (!a->values)) return 13;
  }
  return 0;
}

static int validate_art_io(const yai_law_registry_t* r, const yai_law_artifact_io_t* io, size_t n) {
  for (size_t i = 0; i < n; i++) {
    const yai_law_artifact_io_t* a = &io[i];
    if (!a->role) return 20;

    const yai_law_artifact_role_t* rr = find_role(r, a->role);
    if (!rr) return 21;

    // If schema_ref is present in command, it MUST match registry schema_ref
    if (a->schema_ref && rr->schema_ref && strcmp(a->schema_ref, rr->schema_ref) != 0) return 22;
  }
  return 0;
}

int yai_law_registry_validate_command(const yai_law_registry_t* r, const yai_law_command_t* c) {
  if (!r || !c) return 1;
  if (!c->id || !c->name || !c->group || !c->summary) return 2;

  int rc = validate_args(c);
  if (rc != 0) return rc;

  rc = validate_art_io(r, c->emits_artifacts, c->emits_artifacts_len);
  if (rc != 0) return rc;

  rc = validate_art_io(r, c->consumes_artifacts, c->consumes_artifacts_len);
  if (rc != 0) return rc;

  return 0;
}

int yai_law_registry_validate_all(const yai_law_registry_t* r) {
  if (!r) return 1;

  // validate artifacts table
  for (size_t i = 0; i < r->artifacts_len; i++) {
    const yai_law_artifact_role_t* a = &r->artifacts[i];
    if (!a->role || !a->schema_ref || !a->description) return 3;
    for (size_t j = i + 1; j < r->artifacts_len; j++) {
      const yai_law_artifact_role_t* b = &r->artifacts[j];
      if (b->role && strcmp(a->role, b->role) == 0) return 4;
    }
  }

  // validate commands + unique ids
  for (size_t i = 0; i < r->commands_len; i++) {
    const yai_law_command_t* a = &r->commands[i];

    int rc = yai_law_registry_validate_command(r, a);
    if (rc != 0) return rc;

    for (size_t j = i + 1; j < r->commands_len; j++) {
      const yai_law_command_t* b = &r->commands[j];
      if (b->id && strcmp(a->id, b->id) == 0) return 5;
    }
  }

  return 0;
}
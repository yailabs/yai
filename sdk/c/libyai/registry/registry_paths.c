// src/model/registry/registry_paths.c
#include "yai/sdk/registry/registry_paths.h"

#include <errno.h>
#include <limits.h>

#ifndef PATH_MAX
#define PATH_MAX 4096
#endif
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(_WIN32)
#include <direct.h>
#include <io.h>
#define access _access
#define F_OK 0
#define getcwd _getcwd
#else
#include <unistd.h>
#endif

// ---------------------------- helpers ----------------------------

static void yai_free0(char** p) {
  if (p && *p) {
    free(*p);
    *p = NULL;
  }
}

static int yai_exists_path(const char* path) {
  if (!path || !path[0]) return 0;
  return access(path, F_OK) == 0;
}

// Best-effort: access() only. Good enough for now.
static int yai_exists_dir(const char* path) { return yai_exists_path(path); }
static int yai_exists_file(const char* path) { return yai_exists_path(path); }

static char* yai_strdup0(const char* s) {
  if (!s) return NULL;
  size_t n = strlen(s);
  char* out = (char*)malloc(n + 1);
  if (!out) return NULL;
  memcpy(out, s, n + 1);
  return out;
}

static char* yai_path_join2(const char* a, const char* b) {
  if (!a || !b) return NULL;
  size_t na = strlen(a);
  size_t nb = strlen(b);
  int need_slash = (na > 0 && a[na - 1] != '/');

  size_t n = na + (need_slash ? 1 : 0) + nb;
  char* out = (char*)malloc(n + 1);
  if (!out) return NULL;

  memcpy(out, a, na);
  size_t off = na;
  if (need_slash) out[off++] = '/';
  memcpy(out + off, b, nb);
  out[n] = 0;
  return out;
}

static char* yai_dirname_dup(const char* path) {
  if (!path) return NULL;
  size_t n = strlen(path);
  if (n == 0) return NULL;

  char* tmp = yai_strdup0(path);
  if (!tmp) return NULL;

  // strip trailing slashes
  while (n > 1 && tmp[n - 1] == '/') {
    tmp[n - 1] = 0;
    n--;
  }

  char* slash = strrchr(tmp, '/');
  if (!slash) {
    free(tmp);
    return yai_strdup0(".");
  }

  if (slash == tmp) {
    slash[1] = 0; // "/"
    return tmp;
  }

  *slash = 0;
  return tmp;
}

static char* yai_getcwd_dup(void) {
  char buf[PATH_MAX];
  if (!getcwd(buf, sizeof(buf))) return NULL;
  return yai_strdup0(buf);
}

static int yai_try_set_law_dir_join(yai_law_paths_t* out, const char* base, const char* rel);

static int yai_try_set_law_dir_candidates(yai_law_paths_t* out, const char* base) {
  static const char* rel_candidates[] = {
    "compat/law-export",
    "deps/law"
  };
  size_t i;
  if (!out || !base || !base[0]) return EINVAL;
  for (i = 0; i < sizeof(rel_candidates) / sizeof(rel_candidates[0]); i++) {
    if (yai_try_set_law_dir_join(out, base, rel_candidates[i]) == 0) {
      return 0;
    }
  }
  return ENOENT;
}

static char* yai_find_law_dir_from(const char* start_dir) {
  // Walk upwards and probe compatibility law-export candidates.
  char* cur = yai_strdup0(start_dir);
  if (!cur) return NULL;

  for (;;) {
    size_t i;
    static const char* rel_candidates[] = {
      "compat/law-export",
      "deps/law"
    };
    for (i = 0; i < sizeof(rel_candidates) / sizeof(rel_candidates[0]); i++) {
      char* candidate = yai_path_join2(cur, rel_candidates[i]);
      if (!candidate) {
        free(cur);
        return NULL;
      }
      if (yai_exists_dir(candidate)) {
        free(cur);
        return candidate; // malloc'd
      }
      free(candidate);
    }

    // Stop at filesystem root
    char* parent = yai_dirname_dup(cur);
    if (!parent) {
      free(cur);
      return NULL;
    }

    if (strcmp(parent, cur) == 0) {
      free(parent);
      free(cur);
      return NULL;
    }

    free(cur);
    cur = parent;
  }
}

static int yai_set_law_dir(yai_law_paths_t* out, const char* law_dir) {
  if (!out || !law_dir || !law_dir[0]) return EINVAL;
  if (!yai_exists_dir(law_dir)) return ENOENT;
  out->law_dir = yai_strdup0(law_dir);
  if (!out->law_dir) return ENOMEM;
  return 0;
}

static int yai_try_set_law_dir_join(yai_law_paths_t* out, const char* base, const char* rel) {
  if (!out || !base || !base[0]) return EINVAL;
  char* cand = yai_path_join2(base, rel);
  if (!cand) return ENOMEM;
  if (yai_exists_dir(cand)) {
    out->law_dir = cand;
    return 0;
  }
  free(cand);
  return ENOENT;
}

/* ---------------------------- API ---------------------------- */

static void yai_law_paths_zero(yai_law_paths_t* p) { memset(p, 0, sizeof(*p)); }

static void yai_law_paths_release(yai_law_paths_t* p) {
  yai_free0(&p->law_dir);
  yai_free0(&p->registry_primitives);
  yai_free0(&p->registry_commands);
  yai_free0(&p->registry_artifacts);
  yai_free0(&p->schema_primitives);
  yai_free0(&p->schema_commands);
  yai_free0(&p->schema_artifacts);
  yai_free0(&p->artifacts_schema_dir);
}

int yai_law_paths_init(yai_law_paths_t* out, const char* repo_root_hint) {
  if (!out) return EINVAL;
  yai_law_paths_zero(out);

  /* 1) Environment override: explicit compatibility law directory. */
  const char* env = getenv("YAI_SDK_COMPAT_REGISTRY_DIR");
  if (env && env[0]) {
    int rc = yai_set_law_dir(out, env);
    if (rc != 0) return rc;
  }

  /* 2) SDK-root compatibility probes. */
  /* YAI_SDK_ROOT is injected by the SDK Makefile at compile time. */
  #ifdef YAI_SDK_ROOT
    if (!out->law_dir) {
      const char* sdk_root = YAI_SDK_ROOT; /* compile-time string */
      if (sdk_root && sdk_root[0]) {
        (void)yai_try_set_law_dir_candidates(out, sdk_root);
      }
    }
  #endif

  /* 3) repo_root_hint compatibility probes. */
  if (!out->law_dir && repo_root_hint && repo_root_hint[0]) {
    (void)yai_try_set_law_dir_candidates(out, repo_root_hint);
  }

  /* 4) Fallback probe: walk upward from current working directory. */
  if (!out->law_dir) {
    char* cwd = yai_getcwd_dup();
    if (!cwd) return errno ? errno : ENOENT;
    char* found = yai_find_law_dir_from(cwd);
    free(cwd);
    if (!found) return ENOENT;
    out->law_dir = found;
  }

  /* Build canonical paths under law current layout. */
  char* law_dir = out->law_dir;

  /* Registries */
  out->registry_primitives = yai_path_join2(law_dir, "model/registry/primitives.v1.json");
  out->registry_commands   = yai_path_join2(law_dir, "model/registry/commands.v1.json");
  out->registry_artifacts  = yai_path_join2(law_dir, "model/registry/artifacts.v1.json");

  /* Schemas */
  out->schema_primitives   = yai_path_join2(law_dir, "model/schema/registry/primitives.v1.schema.json");
  out->schema_commands     = yai_path_join2(law_dir, "model/schema/registry/commands.v1.schema.json");
  out->schema_artifacts    = yai_path_join2(law_dir, "model/schema/registry/artifacts.v1.schema.json");

  /* Schema directory */
  out->artifacts_schema_dir = yai_path_join2(law_dir, "model/registry/schema");

  if (!out->registry_primitives || !out->registry_commands || !out->registry_artifacts ||
      !out->schema_primitives || !out->schema_commands || !out->schema_artifacts ||
      !out->artifacts_schema_dir) {
    yai_law_paths_release(out);
    return ENOMEM;
  }

  /* Fail fast: required files and directories must exist. */
  if (!yai_exists_file(out->registry_primitives) ||
      !yai_exists_file(out->registry_commands) ||
      !yai_exists_file(out->registry_artifacts) ||
      !yai_exists_file(out->schema_primitives) ||
      !yai_exists_file(out->schema_commands) ||
      !yai_exists_file(out->schema_artifacts) ||
      !yai_exists_dir(out->artifacts_schema_dir)) {
    yai_law_paths_release(out);
    return ENOENT;
  }

  return 0;
}

void yai_law_paths_free(yai_law_paths_t* p) {
  if (!p) return;
  yai_law_paths_release(p);
  yai_law_paths_zero(p);
}

/* Getters */
const char* yai_law_dir(const yai_law_paths_t* p) { return p ? p->law_dir : NULL; }
const char* yai_law_registry_primitives(const yai_law_paths_t* p) { return p ? p->registry_primitives : NULL; }
const char* yai_law_registry_commands(const yai_law_paths_t* p) { return p ? p->registry_commands : NULL; }
const char* yai_law_registry_artifacts(const yai_law_paths_t* p) { return p ? p->registry_artifacts : NULL; }

const char* yai_law_schema_primitives(const yai_law_paths_t* p) { return p ? p->schema_primitives : NULL; }
const char* yai_law_schema_commands(const yai_law_paths_t* p) { return p ? p->schema_commands : NULL; }
const char* yai_law_schema_artifacts(const yai_law_paths_t* p) { return p ? p->schema_artifacts : NULL; }

const char* yai_law_artifacts_schema_dir(const yai_law_paths_t* p) { return p ? p->artifacts_schema_dir : NULL; }

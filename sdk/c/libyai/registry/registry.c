// src/model/registry/law.c
#include "yai/sdk/registry/registry.h"
#include "yai/sdk/registry/registry_help.h"
#include "yai/sdk/registry/registry_registry.h"

#include <cJSON.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

static const char *resolve_law_root(void) {
  const char *v = getenv("YAI_SDK_COMPAT_REGISTRY_DIR");
  if (v && v[0]) return v;
  v = getenv("YAI_LAW_ROOT");
  if (v && v[0]) return v;
  return "../law";
}

static int read_text_file(const char *path, char **out) {
  FILE *f = NULL;
  long sz = 0;
  size_t n = 0;
  char *buf = NULL;
  if (!path || !out) return 2;
  *out = NULL;
  f = fopen(path, "rb");
  if (!f) return 3;
  if (fseek(f, 0, SEEK_END) != 0) {
    fclose(f);
    return 4;
  }
  sz = ftell(f);
  if (sz < 0) {
    fclose(f);
    return 5;
  }
  if (fseek(f, 0, SEEK_SET) != 0) {
    fclose(f);
    return 6;
  }
  buf = (char *)malloc((size_t)sz + 1u);
  if (!buf) {
    fclose(f);
    return 7;
  }
  n = fread(buf, 1, (size_t)sz, f);
  fclose(f);
  buf[n] = '\0';
  *out = buf;
  return 0;
}

static int load_governable_objects(cJSON **root_doc, cJSON **objects_arr) {
  char path[1024];
  char *json = NULL;
  cJSON *doc = NULL;
  cJSON *objs = NULL;
  int rc;
  if (!root_doc || !objects_arr) return 2;
  *root_doc = NULL;
  *objects_arr = NULL;
  if (snprintf(path, sizeof(path), "%s/model/registry/governable-objects.v1.json", resolve_law_root()) <= 0) return 2;
  rc = read_text_file(path, &json);
  if (rc != 0) {
    fprintf(stderr, "ERR: cannot read governable objects index: %s (rc=%d)\n", path, rc);
    return 3;
  }
  doc = cJSON_Parse(json);
  free(json);
  if (!doc) {
    fprintf(stderr, "ERR: invalid json in governable objects index: %s\n", path);
    return 4;
  }
  objs = cJSON_GetObjectItemCaseSensitive(doc, "objects");
  if (!cJSON_IsArray(objs)) {
    cJSON_Delete(doc);
    fprintf(stderr, "ERR: governable objects index missing array: objects\n");
    return 5;
  }
  *root_doc = doc;
  *objects_arr = objs;
  return 0;
}

static const char *json_s(cJSON *obj, const char *k, const char *fallback) {
  cJSON *n = NULL;
  if (!obj || !k) return fallback;
  n = cJSON_GetObjectItemCaseSensitive(obj, k);
  if (cJSON_IsString(n) && n->valuestring && n->valuestring[0]) return n->valuestring;
  return fallback;
}

static int json_b(cJSON *obj, const char *k, int fallback) {
  cJSON *n = NULL;
  if (!obj || !k) return fallback;
  n = cJSON_GetObjectItemCaseSensitive(obj, k);
  if (cJSON_IsBool(n)) return cJSON_IsTrue(n);
  return fallback;
}

static int cmd_law_list(int argc, char **argv) {
  cJSON *doc = NULL;
  cJSON *arr = NULL;
  const char *filter_kind = NULL;
  int shown = 0;
  int total = 0;
  int rc = load_governable_objects(&doc, &arr);
  if (rc != 0) return rc;
  if (argc >= 2 && argv[1] && argv[1][0]) filter_kind = argv[1];
  printf("YAI Governable Objects\n");
  printf("  Source            %s/model/registry/governable-objects.v1.json\n", resolve_law_root());
  if (filter_kind) printf("  Kind filter       %s\n", filter_kind);
  printf("\n");
  printf("%-44s %-20s %-12s %-8s\n", "Id", "Kind", "Status", "Runtime");
  printf("%-44s %-20s %-12s %-8s\n", "--------------------------------------------", "--------------------", "------------", "--------");
  total = cJSON_GetArraySize(arr);
  for (int i = 0; i < total; i++) {
    cJSON *it = cJSON_GetArrayItem(arr, i);
    const char *id = json_s(it, "id", "-");
    const char *kind = json_s(it, "kind", "-");
    const char *status = json_s(it, "status", "unknown");
    const char *runtime = json_b(it, "runtime_consumable", 0) ? "yes" : "no";
    if (filter_kind && strcmp(kind, filter_kind) != 0) continue;
    printf("%-44s %-20s %-12s %-8s\n", id, kind, status, runtime);
    shown++;
  }
  printf("\n  Objects shown     %d/%d\n", shown, total);
  cJSON_Delete(doc);
  return 0;
}

static int cmd_law_inspect(const char *id) {
  cJSON *doc = NULL;
  cJSON *arr = NULL;
  int rc = load_governable_objects(&doc, &arr);
  if (rc != 0) return rc;
  for (int i = 0; i < cJSON_GetArraySize(arr); i++) {
    cJSON *it = cJSON_GetArrayItem(arr, i);
    const char *cur = json_s(it, "id", "");
    if (strcmp(cur, id) != 0) continue;
    printf("Governable object inspect\n");
    printf("-------------------------\n\n");
    printf("Identity\n");
    printf("  Id                 %s\n", json_s(it, "id", "—"));
    printf("  Name               %s\n", json_s(it, "name", "—"));
    printf("  Kind               %s\n", json_s(it, "kind", "—"));
    printf("  Status             %s\n", json_s(it, "status", "unknown"));
    printf("\nSemantics\n");
    printf("  Attachment modes   %s\n", json_s(it, "attachment_modes", "none"));
    printf("  Precedence class   %s\n", json_s(it, "precedence_class", "none"));
    printf("  Runtime consumable %s\n", json_b(it, "runtime_consumable", 0) ? "yes" : "no");
    printf("  CLI discoverable   %s\n", json_b(it, "cli_discoverable", 0) ? "yes" : "no");
    printf("  Experimental       %s\n", json_b(it, "experimental", 0) ? "yes" : "no");
    printf("  Deprecated         %s\n", json_b(it, "deprecated", 0) ? "yes" : "no");
    printf("\nReferences\n");
    printf("  Source ref         %s\n", json_s(it, "source_ref", "—"));
    printf("  Manifest ref       %s\n", json_s(it, "manifest_ref", "—"));
    printf("  Compatibility ref  %s\n", json_s(it, "compatibility_ref", "—"));
    cJSON_Delete(doc);
    return 0;
  }
  cJSON_Delete(doc);
  fprintf(stderr, "ERR: unknown object id: %s\n", id ? id : "");
  return 3;
}

static int run_govern_surface(int argc, char **argv) {
  char cli_path[1024];
  char **child_argv = NULL;
  pid_t pid;
  int status = 0;
  int rc = 0;

  if (snprintf(cli_path, sizeof(cli_path), "%s/tools/bin/yai-govern", resolve_law_root()) <= 0) {
    fprintf(stderr, "ERR: unable to resolve governance shell path\n");
    return 2;
  }
  if (access(cli_path, X_OK) != 0) {
    fprintf(stderr, "ERR: governance shell not executable: %s\n", cli_path);
    fprintf(stderr, "Hint: chmod +x %s\n", cli_path);
    return 3;
  }

  child_argv = (char **)calloc((size_t)argc + 2u, sizeof(char *));
  if (!child_argv) {
    fprintf(stderr, "ERR: out of memory\n");
    return 4;
  }
  child_argv[0] = cli_path;
  for (int i = 0; i < argc; i++) {
    child_argv[i + 1] = argv[i];
  }
  child_argv[argc + 1] = NULL;

  pid = fork();
  if (pid < 0) {
    free(child_argv);
    fprintf(stderr, "ERR: cannot fork governance shell\n");
    return 5;
  }
  if (pid == 0) {
    execv(cli_path, child_argv);
    _exit(127);
  }

  free(child_argv);
  if (waitpid(pid, &status, 0) < 0) {
    fprintf(stderr, "ERR: governance shell wait failed\n");
    return 6;
  }
  if (WIFEXITED(status)) {
    rc = WEXITSTATUS(status);
    return rc;
  }
  if (WIFSIGNALED(status)) {
    fprintf(stderr, "ERR: governance shell terminated by signal %d\n", WTERMSIG(status));
    return 7;
  }
  return 7;
}

int yai_cmd_law(int argc, char** argv) {
  // usage:
  //   yai law help [token]
  //   yai law validate [registry|objects|all]
  //   yai law list [kind]
  //   yai law inspect <object-id>
  //   yai law gov <pipeline-command> ...
  if (argc < 1) {
    printf("law\n\n");
    printf("Commands:\n");
    printf("  help [token]                 print registry help\n");
    printf("  list [kind]                  list governable objects\n");
    printf("  inspect <object-id>          inspect one governable object\n");
    printf("  validate [registry|objects|all]\n");
    printf("  gov <pipeline-command> ...   deterministic governance parsing/authoring surface\n");
    printf("\nGov pipeline examples:\n");
    printf("  yai law gov source import --path control/ingestion/sources\n");
    printf("  yai law gov source list\n");
    printf("  yai law gov parse <source-id>\n");
    printf("  yai law gov normalize <parsed-id>\n");
    printf("  yai law gov build <normalized-id>\n");
    printf("  yai law gov candidate list\n");
    printf("  yai law gov validate <candidate-id>\n");
    return 0;
  }

  const char* sub = argv[0];

  if (strcmp(sub, "help") == 0) {
    const char* tok = (argc >= 2) ? argv[1] : NULL;
    if (!tok || !tok[0]) return yai_law_help_print_global();
    return yai_law_help_print_any(tok);
  }

  if (strcmp(sub, "validate") == 0) {
    const char *target = (argc >= 2 && argv[1] && argv[1][0]) ? argv[1] : "all";
    int wants_registry = (strcmp(target, "all") == 0 || strcmp(target, "registry") == 0);
    int wants_objects = (strcmp(target, "all") == 0 || strcmp(target, "objects") == 0);
    if (!wants_registry && !wants_objects) {
      fprintf(stderr, "ERR: unknown validate target: %s\n", target);
      return 2;
    }
    if (wants_registry) {
      if (yai_law_registry_init() != 0) return 2;
      {
        const yai_law_registry_t* r = yai_law_registry();
        if (!r) return 3;
        printf("OK: law registry loaded (commands=%zu artifacts=%zu)\n", r->commands_len, r->artifacts_len);
      }
    }
    if (wants_objects) {
      cJSON *doc = NULL;
      cJSON *arr = NULL;
      int rc = load_governable_objects(&doc, &arr);
      if (rc != 0) return rc;
      printf("OK: governable objects index loaded (objects=%d)\n", cJSON_GetArraySize(arr));
      cJSON_Delete(doc);
    }
    return 0;
  }

  if (strcmp(sub, "list") == 0) {
    return cmd_law_list(argc, argv);
  }

  if (strcmp(sub, "inspect") == 0) {
    if (argc < 2 || !argv[1] || !argv[1][0]) {
      fprintf(stderr, "ERR: missing object id\n");
      fprintf(stderr, "Hint: yai law inspect <object-id>\n");
      return 2;
    }
    return cmd_law_inspect(argv[1]);
  }

  if (strcmp(sub, "gov") == 0) {
    if (argc < 2) {
      fprintf(stderr, "ERR: missing governance pipeline command\n");
      fprintf(stderr, "Hint: yai law gov source list\n");
      return 2;
    }
    return run_govern_surface(argc - 1, &argv[1]);
  }

  // fallback: treat as token
  return yai_law_help_print_any(sub);
}

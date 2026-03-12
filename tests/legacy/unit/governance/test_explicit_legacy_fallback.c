#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <sys/stat.h>
#include <unistd.h>

#include <yai/policy/governance/loader.h>

int main(void) {
  yai_governance_runtime_t rt;
  char err[256] = {0};
  char tmp[256];
  char dir[512];
  char file[768];
  char out[256];
  FILE *f;

  if (yai_governance_load_runtime(&rt, err, sizeof(err)) != 0) {
    fprintf(stderr, "explicit_legacy_fallback: runtime load failed: %s\n", err);
    return 1;
  }

  if (snprintf(tmp, sizeof(tmp), "/tmp/yai_legacy_fallback_%ld", (long)getpid()) <= 0) return 1;
  if (mkdir(tmp, 0700) != 0) {
    /* Allow reruns in same process environment. */
    if (errno != EEXIST) {
      fprintf(stderr, "explicit_legacy_fallback: mkdir tmp failed\n");
      return 1;
    }
  }
  if (snprintf(dir, sizeof(dir), "%s/legacy-only", tmp) <= 0) return 1;
  if (mkdir(dir, 0700) != 0 && errno != EEXIST) {
    fprintf(stderr, "explicit_legacy_fallback: mkdir legacy-only failed\n");
    return 1;
  }
  if (snprintf(file, sizeof(file), "%s/flag.json", dir) <= 0) return 1;
  f = fopen(file, "wb");
  if (!f) {
    fprintf(stderr, "explicit_legacy_fallback: create file failed\n");
    return 1;
  }
  (void)fputs("{\"legacy\":\"ok\"}\n", f);
  fclose(f);

  (void)snprintf(rt.root, sizeof(rt.root), "%s", tmp);
  unsetenv("YAI_GOVERNANCE_CANONICAL_ONLY");
  unsetenv("YAI_GOVERNANCE_ALLOW_LEGACY");

  if (yai_governance_read_surface_json(&rt, "legacy-only/flag.json", out, sizeof(out)) == 0) {
    fprintf(stderr, "explicit_legacy_fallback: legacy read unexpectedly enabled by default\n");
    return 1;
  }

  if (setenv("YAI_GOVERNANCE_ALLOW_LEGACY", "1", 1) != 0) {
    fprintf(stderr, "explicit_legacy_fallback: setenv failed\n");
    return 1;
  }
  if (yai_governance_read_surface_json(&rt, "legacy-only/flag.json", out, sizeof(out)) != 0) {
    fprintf(stderr, "explicit_legacy_fallback: legacy read not enabled when explicit\n");
    return 1;
  }
  if (strstr(out, "\"legacy\":\"ok\"") == NULL) {
    fprintf(stderr, "explicit_legacy_fallback: unexpected payload\n");
    return 1;
  }

  puts("explicit_legacy_fallback: ok");
  return 0;
}

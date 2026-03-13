#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <yai/api/version.h>
#include <yai/data/binding.h>
#include <yai/data/query.h>

static void print_help(void)
{
  puts("yai-datad - canonical L2 data service entrypoint");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage:");
  puts("  yai-datad summary <data-scope-id>");
  puts("  yai-datad operational-summary <data-scope-id>");
  puts("  yai-datad count <data-scope-id> <record-class>");
  puts("  yai-datad tail <data-scope-id> <record-class> [limit]");
}

static int parse_limit(const char *in, size_t *out)
{
  unsigned long v;
  char *end = NULL;
  if (!in || !out) return -1;
  v = strtoul(in, &end, 10);
  if (end == in || (end && *end != '\0')) return -1;
  if (v == 0) return -1;
  *out = (size_t)v;
  return 0;
}

int main(int argc, char **argv)
{
  char out_json[8192];
  char err[128];
  size_t out_count = 0;
  size_t limit = 16;

  if (argc < 2 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "help") == 0) {
    print_help();
    return argc < 2 ? 1 : 0;
  }

  out_json[0] = '\0';
  err[0] = '\0';
  if (yai_data_store_binding_init(err, sizeof(err)) != 0) {
    fprintf(stderr, "yai-datad: binding init failed: %s\n", err[0] ? err : "unknown");
    return 70;
  }

  if (strcmp(argv[1], "summary") == 0 && argc == 3) {
    if (yai_data_query_summary_json(argv[2], out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      fprintf(stderr, "yai-datad: summary failed: %s\n", err[0] ? err : "unknown");
      return 1;
    }
    puts(out_json);
    return 0;
  }

  if (strcmp(argv[1], "operational-summary") == 0 && argc == 3) {
    if (yai_data_query_operational_summary_json(argv[2], out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      fprintf(stderr, "yai-datad: operational-summary failed: %s\n", err[0] ? err : "unknown");
      return 1;
    }
    puts(out_json);
    return 0;
  }

  if (strcmp(argv[1], "count") == 0 && argc == 4) {
    if (yai_data_query_count(argv[2], argv[3], &out_count, err, sizeof(err)) != 0) {
      fprintf(stderr, "yai-datad: count failed: %s\n", err[0] ? err : "unknown");
      return 1;
    }
    printf("%zu\n", out_count);
    return 0;
  }

  if (strcmp(argv[1], "tail") == 0 && (argc == 4 || argc == 5)) {
    if (argc == 5 && parse_limit(argv[4], &limit) != 0) {
      fprintf(stderr, "yai-datad: invalid limit\n");
      return 64;
    }
    if (yai_data_query_tail_json(argv[2], argv[3], limit, out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      fprintf(stderr, "yai-datad: tail failed: %s\n", err[0] ? err : "unknown");
      return 1;
    }
    puts(out_json);
    return 0;
  }

  print_help();
  return 64;
}

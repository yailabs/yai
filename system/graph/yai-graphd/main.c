#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <string.h>

#include <yai/api/version.h>
#include <yai/graph/lineage.h>
#include <yai/graph/query.h>
#include <yai/cognition/memory.h>

static void print_help(void)
{
  puts("yai-graphd - canonical L2 graph service entrypoint");
  printf("version: %s\n", YAI_VERSION_STRING);
  puts("");
  puts("usage:");
  puts("  yai-graphd summary <graph-scope-id>");
  puts("  yai-graphd unified-summary <graph-scope-id>");
  puts("  yai-graphd lineage <graph-scope-id> <anchor-ref>");
}

static int print_graph_error(const char *op, const char *err)
{
  fprintf(stderr, "yai-graphd: %s failed: %s\n", op, (err && err[0]) ? err : "unknown");
  return 1;
}

int main(int argc, char **argv)
{
  char out_json[4096];
  char err[128];
  int rc;

  if (argc < 2 || strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "help") == 0) {
    print_help();
    return argc < 2 ? 1 : 0;
  }

  rc = yai_knowledge_memory_start();
  if (rc != YAI_MIND_OK) {
    fprintf(stderr, "yai-graphd: memory start failed rc=%d\n", rc);
    return 70;
  }

  out_json[0] = '\0';
  err[0] = '\0';

  if (strcmp(argv[1], "summary") == 0 && argc == 3) {
    if (yai_graph_query_summary(argv[2], out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      (void)yai_knowledge_memory_stop();
      return print_graph_error("summary", err);
    }
    puts(out_json);
    (void)yai_knowledge_memory_stop();
    return 0;
  }

  if (strcmp(argv[1], "unified-summary") == 0 && argc == 3) {
    if (yai_graph_query_unified_summary(argv[2], out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      (void)yai_knowledge_memory_stop();
      return print_graph_error("unified-summary", err);
    }
    puts(out_json);
    (void)yai_knowledge_memory_stop();
    return 0;
  }

  if (strcmp(argv[1], "lineage") == 0 && argc == 4) {
    if (yai_graph_lineage_summary(argv[2], argv[3], out_json, sizeof(out_json), err, sizeof(err)) != 0) {
      (void)yai_knowledge_memory_stop();
      return print_graph_error("lineage", err);
    }
    puts(out_json);
    (void)yai_knowledge_memory_stop();
    return 0;
  }

  print_help();
  (void)yai_knowledge_memory_stop();
  return 64;
}

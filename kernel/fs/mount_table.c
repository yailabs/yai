#include <string.h>

#include <yai/fs/mount_table.h>

static yai_mount_handle_t g_next_mount_handle = 1u;

void yai_mount_table_defaults(yai_mount_table_t *table) {
  if (!table) {
    return;
  }
  memset(table, 0, sizeof(*table));
}

int yai_mount_table_attach(yai_mount_table_t *table,
                           const yai_mount_entry_t *entry) {
  yai_mount_entry_t normalized;

  if (!table || !entry || entry->target[0] == '\0') {
    return -1;
  }
  if (table->len >= YAI_FS_MOUNT_MAX) {
    return -1;
  }

  normalized = *entry;
  if (normalized.handle == 0) {
    normalized.handle = g_next_mount_handle++;
  }

  table->entries[table->len++] = normalized;
  return 0;
}

int yai_mount_table_lookup(const yai_mount_table_t *table,
                           const char *target,
                           yai_mount_entry_t *out_entry) {
  size_t i;

  if (!table || !target || target[0] == '\0') {
    return -1;
  }

  for (i = 0; i < table->len; ++i) {
    if (strcmp(table->entries[i].target, target) == 0) {
      if (out_entry) {
        *out_entry = table->entries[i];
      }
      return 0;
    }
  }

  return -1;
}

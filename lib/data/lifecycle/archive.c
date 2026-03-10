/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/data/archive.h>

#include <errno.h>
#if defined(YAI_HAVE_LMDB)
#include <lmdb.h>
#endif
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

#include "../store/internal.h"

static int mkdir_if_needed(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return -1;
  if (mkdir(path, 0775) == 0) return 0;
  if (errno == EEXIST) {
    if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
  }
  return -1;
}

int yai_data_archive_rotate_class(const char *workspace_id,
                                  const char *record_class,
                                  char *out_archive_path,
                                  size_t out_archive_path_cap,
                                  char *err,
                                  size_t err_cap)
{
  char data_dir[768];
  char archive_dir[768];
  char archived[900];
  FILE *out = NULL;
  long ts = (long)time(NULL);
#if defined(YAI_HAVE_LMDB)
  MDB_env *env = NULL;
  MDB_txn *txn = NULL;
  MDB_cursor *cursor = NULL;
  MDB_dbi dbi;
  MDB_val key;
  MDB_val val;
  char lmdb_dir[800];
  int rc;
#endif
  if (out_archive_path && out_archive_path_cap > 0) out_archive_path[0] = '\0';
  if (err && err_cap > 0) err[0] = '\0';
  if (yai_data_store_paths(workspace_id, record_class, data_dir, sizeof(data_dir), NULL, 0, err, err_cap) != 0) return -1;
  if (snprintf(archive_dir, sizeof(archive_dir), "%s/archive", data_dir) <= 0 ||
      snprintf(archived, sizeof(archived), "%s/%s-%ld.jsonl", archive_dir, record_class, ts) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "archive_path_format_failed");
    return -1;
  }
  if (mkdir_if_needed(archive_dir) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "archive_dir_mkdir_failed");
    return -1;
  }
  out = fopen(archived, "w");
  if (!out) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "archive_open_failed");
    return -1;
  }
#if !defined(YAI_HAVE_LMDB)
  fclose(out);
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_not_available");
  return -1;
#else
  if (snprintf(lmdb_dir, sizeof(lmdb_dir), "%s/lmdb", data_dir) <= 0) {
    fclose(out);
    return -1;
  }
  if (mdb_env_create(&env) != 0) {
    fclose(out);
    return -1;
  }
  (void)mdb_env_set_mapsize(env, 128UL * 1024UL * 1024UL);
  (void)mdb_env_set_maxdbs(env, 16);
  if (mdb_env_open(env, lmdb_dir, 0, 0664) != 0) {
    mdb_env_close(env);
    fclose(out);
    return 0;
  }
  if (mdb_txn_begin(env, NULL, 0, &txn) != 0) {
    mdb_env_close(env);
    fclose(out);
    return -1;
  }
  rc = mdb_dbi_open(txn, record_class, MDB_CREATE, &dbi);
  if (rc != 0) {
    mdb_txn_abort(txn);
    mdb_env_close(env);
    fclose(out);
    return 0;
  }
  rc = mdb_cursor_open(txn, dbi, &cursor);
  if (rc == 0) {
    for (rc = mdb_cursor_get(cursor, &key, &val, MDB_FIRST);
         rc == 0;
         rc = mdb_cursor_get(cursor, &key, &val, MDB_NEXT)) {
      (void)fwrite(val.mv_data, 1, val.mv_size, out);
      (void)fwrite("\n", 1, 1, out);
      (void)mdb_cursor_del(cursor, 0);
    }
    mdb_cursor_close(cursor);
  }
  fclose(out);
  if (mdb_txn_commit(txn) != 0) {
    mdb_env_close(env);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "archive_commit_failed");
    return -1;
  }
  mdb_env_close(env);
#endif
  if (out_archive_path && out_archive_path_cap > 0) snprintf(out_archive_path, out_archive_path_cap, "%s", archived);
  return 0;
}

/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/data/retention.h>

#if defined(YAI_HAVE_LMDB)
#include <lmdb.h>
#include <errno.h>
#include <stdio.h>
#include <sys/stat.h>
#endif

#include "../internal/store_internal.h"

#if defined(YAI_HAVE_LMDB)
static int mkdir_if_needed(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return -1;
  if (mkdir(path, 0775) == 0) return 0;
  if (errno == EEXIST && stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
  return -1;
}

static int open_env(const char *workspace_id, MDB_env **env_out, char *err, size_t err_cap)
{
  MDB_env *env = NULL;
  char data_dir[768];
  char env_dir[768];
  if (yai_data_store_paths(workspace_id, "events", data_dir, sizeof(data_dir), NULL, 0, err, err_cap) != 0) return -1;
  if (snprintf(env_dir, sizeof(env_dir), "%s/lmdb", data_dir) <= 0) return -1;
  if (mkdir_if_needed(data_dir) != 0 || mkdir_if_needed(env_dir) != 0) return -1;
  if (mdb_env_create(&env) != 0) return -1;
  (void)mdb_env_set_mapsize(env, 128UL * 1024UL * 1024UL);
  (void)mdb_env_set_maxdbs(env, 16);
  if (mdb_env_open(env, env_dir, 0, 0664) != 0) {
    mdb_env_close(env);
    return -1;
  }
  *env_out = env;
  return 0;
}
#endif

int yai_data_retention_prune_tail(const char *workspace_id,
                                  const char *record_class,
                                  size_t keep_last,
                                  size_t *pruned_out,
                                  char *err,
                                  size_t err_cap)
{
#if !defined(YAI_HAVE_LMDB)
  (void)workspace_id; (void)record_class; (void)keep_last;
  if (pruned_out) *pruned_out = 0;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_not_available");
  return -1;
#else
  MDB_env *env = NULL;
  MDB_txn *txn = NULL;
  MDB_cursor *cursor = NULL;
  MDB_dbi dbi;
  MDB_val key;
  MDB_val val;
  int rc;
  size_t total = 0;
  size_t to_prune;
  if (pruned_out) *pruned_out = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !record_class || !record_class[0]) return -1;
  if (keep_last == 0) keep_last = 1;
  if (open_env(workspace_id, &env, err, err_cap) != 0) return 0;
  rc = mdb_txn_begin(env, NULL, 0, &txn);
  if (rc != 0) {
    mdb_env_close(env);
    return -1;
  }
  rc = mdb_dbi_open(txn, record_class, MDB_CREATE, &dbi);
  if (rc != 0) {
    mdb_txn_abort(txn);
    mdb_env_close(env);
    return 0;
  }
  rc = mdb_cursor_open(txn, dbi, &cursor);
  if (rc != 0) {
    mdb_txn_abort(txn);
    mdb_env_close(env);
    return -1;
  }
  for (rc = mdb_cursor_get(cursor, &key, &val, MDB_FIRST);
       rc == 0;
       rc = mdb_cursor_get(cursor, &key, &val, MDB_NEXT)) {
    total++;
  }
  if (total <= keep_last) {
    mdb_cursor_close(cursor);
    mdb_txn_abort(txn);
    mdb_env_close(env);
    return 0;
  }
  to_prune = total - keep_last;
  rc = mdb_cursor_get(cursor, &key, &val, MDB_FIRST);
  while (rc == 0 && to_prune > 0) {
    rc = mdb_cursor_del(cursor, 0);
    if (rc != 0) break;
    to_prune--;
    rc = mdb_cursor_get(cursor, &key, &val, MDB_NEXT);
  }
  mdb_cursor_close(cursor);
  if (mdb_txn_commit(txn) != 0) {
    mdb_env_close(env);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "retention_commit_failed");
    return -1;
  }
  mdb_env_close(env);
  if (pruned_out) *pruned_out = total - keep_last - to_prune;
  return 0;
#endif
}

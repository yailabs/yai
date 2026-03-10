/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include <yai/data/store.h>
#include <yai/data/binding.h>

#include <errno.h>
#if defined(YAI_HAVE_LMDB)
#include <lmdb.h>
#endif
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

#include "internal.h"

#if defined(YAI_HAVE_HIREDIS)
#include <hiredis/hiredis.h>
#endif

static unsigned int g_record_seq = 0;

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

static int yai_lmdb_open_env(const char *workspace_id,
                             int create,
                             MDB_env **env_out,
                             char *env_path,
                             size_t env_path_cap,
                             char *err,
                             size_t err_cap)
{
#if !defined(YAI_HAVE_LMDB)
  (void)workspace_id; (void)create; (void)env_out; (void)env_path; (void)env_path_cap;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_not_available");
  return -1;
#else
  MDB_env *env = NULL;
  char base_dir[768];
  if (err && err_cap > 0) err[0] = '\0';
  if (!env_out || !env_path || env_path_cap == 0) return -1;
  if (yai_data_store_paths(workspace_id, "events", base_dir, sizeof(base_dir), NULL, 0, err, err_cap) != 0) return -1;
  if (snprintf(env_path, env_path_cap, "%s/lmdb", base_dir) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_path_format_failed");
    return -1;
  }
  if (create && mkdir_if_needed(base_dir) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "data_dir_mkdir_failed");
    return -1;
  }
  if (create && mkdir_if_needed(env_path) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_dir_mkdir_failed");
    return -1;
  }
  if (mdb_env_create(&env) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_env_create_failed");
    return -1;
  }
  (void)mdb_env_set_mapsize(env, 128UL * 1024UL * 1024UL);
  (void)mdb_env_set_maxdbs(env, 16);
  if (mdb_env_open(env, env_path, 0, 0664) != 0) {
    mdb_env_close(env);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_env_open_failed");
    return -1;
  }
  *env_out = env;
  return 0;
#endif
}

static int yai_lmdb_open_class_db(MDB_txn *txn, const char *record_class, MDB_dbi *dbi_out)
{
#if !defined(YAI_HAVE_LMDB)
  (void)txn; (void)record_class; (void)dbi_out;
  return -1;
#else
  return mdb_dbi_open(txn, record_class, MDB_CREATE, dbi_out);
#endif
}

static void yai_redis_mirror_ref(const char *workspace_id, const char *record_class, const char *ref)
{
#if defined(YAI_HAVE_HIREDIS)
  redisContext *ctx;
  redisReply *reply = NULL;
  const char *enable = getenv("YAI_REDIS_ENABLE");
  const char *host = getenv("YAI_REDIS_HOST");
  const char *port_env = getenv("YAI_REDIS_PORT");
  int port = 6379;
  char key[192];
  if (!enable || strcmp(enable, "1") != 0) return;
  if (port_env && port_env[0]) port = atoi(port_env);
  if (!host || !host[0]) host = "127.0.0.1";
  ctx = redisConnect(host, port);
  if (!ctx || ctx->err) {
    if (ctx) redisFree(ctx);
    return;
  }
  (void)snprintf(key, sizeof(key), "yai:%s:%s:refs", workspace_id, record_class);
  reply = redisCommand(ctx, "LPUSH %s %s", key, ref ? ref : "");
  if (reply) freeReplyObject(reply);
  reply = redisCommand(ctx, "LTRIM %s 0 255", key);
  if (reply) freeReplyObject(reply);
  redisFree(ctx);
#else
  (void)workspace_id; (void)record_class; (void)ref;
#endif
}

int yai_data_store_paths(const char *workspace_id,
                         const char *record_class,
                         char *dir_out,
                         size_t dir_cap,
                         char *file_out,
                         size_t file_cap,
                         char *err,
                         size_t err_cap)
{
  char ws_root[768];
  const char *base = yai_data_store_binding_root();
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !record_class || !record_class[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_or_class_missing");
    return -1;
  }
  if (!base || !base[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "store_binding_not_initialized");
    return -1;
  }
  if (snprintf(ws_root, sizeof(ws_root), "%s/%s", base, workspace_id) <= 0 ||
      (dir_out && dir_cap > 0 && snprintf(dir_out, dir_cap, "%s/data", ws_root) <= 0) ||
      (file_out && file_cap > 0 && snprintf(file_out, file_cap, "%s/data/%s.jsonl", ws_root, record_class) <= 0)) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "path_format_failed");
    return -1;
  }
  return 0;
}

int yai_data_store_init_workspace(const char *workspace_id, char *err, size_t err_cap)
{
  char dir[768];
  char path_err[96];
  if (err && err_cap > 0) err[0] = '\0';
  if (yai_data_store_binding_is_ready() == 0 &&
      yai_data_store_binding_init(err, err_cap) != 0) {
    return -1;
  }
  if (yai_data_store_binding_attach_workspace(workspace_id, err, err_cap) != 0) return -1;
  if (yai_data_store_paths(workspace_id, "events", dir, sizeof(dir), NULL, 0, path_err, sizeof(path_err)) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", path_err);
    return -1;
  }
  if (mkdir_if_needed(dir) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "workspace_data_dir_mkdir_failed");
    return -1;
  }
  return 0;
}

int yai_data_store_append(const char *workspace_id,
                          const char *record_class,
                          const char *record_json,
                          char *out_ref,
                          size_t out_ref_cap,
                          char *err,
                          size_t err_cap)
{
#if !defined(YAI_HAVE_LMDB)
  (void)workspace_id;
  (void)record_class;
  (void)record_json;
  (void)out_ref;
  (void)out_ref_cap;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_not_available");
  return -1;
#else
  MDB_env *env = NULL;
  MDB_txn *txn = NULL;
  MDB_dbi dbi;
  MDB_val key;
  MDB_val val;
  char env_path[768];
  char key_buf[64];
  long ts = (long)time(NULL);
  int rc;
  if (err && err_cap > 0) err[0] = '\0';
  if (out_ref && out_ref_cap > 0) out_ref[0] = '\0';
  if (!record_json || !record_json[0]) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "record_json_missing");
    return -1;
  }
  if (yai_data_store_init_workspace(workspace_id, err, err_cap) != 0) return -1;
  if (yai_lmdb_open_env(workspace_id, 1, &env, env_path, sizeof(env_path), err, err_cap) != 0) return -1;
  rc = mdb_txn_begin(env, NULL, 0, &txn);
  if (rc != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "lmdb_txn_begin_failed:%d", rc);
    mdb_env_close(env);
    return -1;
  }
  rc = yai_lmdb_open_class_db(txn, record_class, &dbi);
  if (rc != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "lmdb_dbi_open_failed:%d", rc);
    mdb_txn_abort(txn);
    mdb_env_close(env);
    return -1;
  }
  g_record_seq++;
  (void)snprintf(key_buf, sizeof(key_buf), "%020ld-%08u", ts, g_record_seq);
  key.mv_data = key_buf;
  key.mv_size = strlen(key_buf);
  val.mv_data = (void *)record_json;
  val.mv_size = strlen(record_json);
  rc = mdb_put(txn, dbi, &key, &val, 0);
  if (rc != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "lmdb_put_failed:%d", rc);
    mdb_txn_abort(txn);
    mdb_env_close(env);
    return -1;
  }
  rc = mdb_txn_commit(txn);
  if (rc != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "lmdb_txn_commit_failed:%d", rc);
    mdb_env_close(env);
    return -1;
  }
  mdb_env_close(env);
  if (out_ref && out_ref_cap > 0) {
    snprintf(out_ref, out_ref_cap, "dp:%s:%s:%s", workspace_id, record_class, key_buf);
    yai_redis_mirror_ref(workspace_id, record_class, out_ref);
  }
  {
    char duck_err[96];
    int duck_rc = yai_data_duckdb_append(workspace_id, record_class, key_buf, record_json, duck_err, sizeof(duck_err));
    (void)duck_rc;
  }
  return 0;
#endif
}

int yai_data_store_count_lines(const char *path, size_t *out_count, char *err, size_t err_cap)
{
  FILE *f;
  int ch;
  size_t count = 0;
  if (err && err_cap > 0) err[0] = '\0';
  if (out_count) *out_count = 0;
  if (!path || !path[0] || !out_count) return -1;
  f = fopen(path, "r");
  if (!f) {
    if (errno == ENOENT) {
      *out_count = 0;
      return 0;
    }
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "store_open_failed");
    return -1;
  }
  while ((ch = fgetc(f)) != EOF) {
    if (ch == '\n') count++;
  }
  fclose(f);
  *out_count = count;
  return 0;
}

int yai_data_store_tail_json(const char *workspace_id,
                             const char *record_class,
                             size_t limit,
                             char *out_json,
                             size_t out_cap,
                             char *err,
                             size_t err_cap)
{
#if !defined(YAI_HAVE_LMDB)
  (void)workspace_id;
  (void)record_class;
  (void)limit;
  if (!out_json || out_cap == 0) return -1;
  out_json[0] = '\0';
  if (snprintf(out_json, out_cap, "[]") <= 0) return -1;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "lmdb_not_available");
  return 0;
#else
  MDB_env *env = NULL;
  MDB_txn *txn = NULL;
  MDB_cursor *cursor = NULL;
  MDB_dbi dbi;
  MDB_val key;
  MDB_val val;
  char env_path[768];
  char ring[64][2048];
  size_t ring_cap = 64;
  size_t idx = 0;
  size_t total = 0;
  size_t i;
  int rc;
  if (err && err_cap > 0) err[0] = '\0';
  if (!out_json || out_cap == 0) return -1;
  out_json[0] = '\0';
  if (limit == 0) limit = 16;
  if (limit > ring_cap) limit = ring_cap;
  if (yai_lmdb_open_env(workspace_id, 0, &env, env_path, sizeof(env_path), err, err_cap) != 0) {
    snprintf(out_json, out_cap, "[]");
    return 0;
  }
  rc = mdb_txn_begin(env, NULL, MDB_RDONLY, &txn);
  if (rc != 0) {
    mdb_env_close(env);
    if (err && err_cap > 0) snprintf(err, err_cap, "lmdb_txn_begin_failed:%d", rc);
    return -1;
  }
  rc = yai_lmdb_open_class_db(txn, record_class, &dbi);
  if (rc != 0) {
    mdb_txn_abort(txn);
    mdb_env_close(env);
    snprintf(out_json, out_cap, "[]");
    return 0;
  }
  rc = mdb_cursor_open(txn, dbi, &cursor);
  if (rc == 0) {
    for (rc = mdb_cursor_get(cursor, &key, &val, MDB_FIRST);
         rc == 0;
         rc = mdb_cursor_get(cursor, &key, &val, MDB_NEXT)) {
      size_t at = idx % limit;
      size_t vlen = val.mv_size < sizeof(ring[at]) - 1 ? val.mv_size : sizeof(ring[at]) - 1;
      memcpy(ring[at], val.mv_data, vlen);
      ring[at][vlen] = '\0';
      idx++;
      total++;
    }
    mdb_cursor_close(cursor);
  }
  mdb_txn_abort(txn);
  mdb_env_close(env);
  if (snprintf(out_json, out_cap, "[") <= 0) return -1;
  {
    size_t start = (total > limit) ? (total - limit) : 0;
    int first = 1;
    for (i = start; i < total; i++) {
      size_t at = i % limit;
      int n;
      n = snprintf(out_json + strlen(out_json),
                   out_cap - strlen(out_json),
                   "%s%s",
                   first ? "" : ",",
                   ring[at]);
      if (n <= 0 || (size_t)n >= (out_cap - strlen(out_json))) return -1;
      first = 0;
    }
  }
  if (snprintf(out_json + strlen(out_json), out_cap - strlen(out_json), "]") <= 0) return -1;
  return 0;
#endif
}

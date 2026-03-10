/* SPDX-License-Identifier: Apache-2.0 */

#define _POSIX_C_SOURCE 200809L

#include "internal.h"

#include <stdio.h>
#include <string.h>
#include <sys/stat.h>

#include <yai/data/binding.h>

#if defined(YAI_HAVE_DUCKDB)
#include <duckdb.h>
#endif

static int mkdir_if_needed(const char *path)
{
  struct stat st;
  if (!path || !path[0]) return -1;
  if (mkdir(path, 0775) == 0) return 0;
  if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) return 0;
  return -1;
}

static int yai_data_duckdb_path(const char *workspace_id, char *out, size_t out_cap, char *err, size_t err_cap)
{
  char data_dir[768];
  if (err && err_cap > 0) err[0] = '\0';
  if (yai_data_store_paths(workspace_id, "events", data_dir, sizeof(data_dir), NULL, 0, err, err_cap) != 0) return -1;
  if (mkdir_if_needed(data_dir) != 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_data_dir_mkdir_failed");
    return -1;
  }
  if (snprintf(out, out_cap, "%s/data_plane.duckdb", data_dir) <= 0) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_path_format_failed");
    return -1;
  }
  return 0;
}

int yai_data_duckdb_append(const char *workspace_id,
                           const char *record_class,
                           const char *record_key,
                           const char *record_json,
                           char *err,
                           size_t err_cap)
{
#if defined(YAI_HAVE_DUCKDB)
  duckdb_database db = NULL;
  duckdb_connection conn = NULL;
  duckdb_prepared_statement stmt = NULL;
  duckdb_state st;
  char path[800];
  if (err && err_cap > 0) err[0] = '\0';
  if (!workspace_id || !workspace_id[0] || !record_class || !record_key || !record_json) return -1;
  if (yai_data_duckdb_path(workspace_id, path, sizeof(path), err, err_cap) != 0) return -1;
  if (duckdb_open(path, &db) != DuckDBSuccess) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_open_failed");
    return -1;
  }
  if (duckdb_connect(db, &conn) != DuckDBSuccess) {
    duckdb_close(&db);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_connect_failed");
    return -1;
  }
  (void)duckdb_query(conn,
                     "CREATE TABLE IF NOT EXISTS yai_records("
                     "workspace_id VARCHAR, record_class VARCHAR, record_key VARCHAR, payload JSON, created_at TIMESTAMP DEFAULT current_timestamp)",
                     NULL);
  st = duckdb_prepare(conn,
                      "INSERT INTO yai_records(workspace_id,record_class,record_key,payload) VALUES (?,?,?,?::JSON)",
                      &stmt);
  if (st != DuckDBSuccess) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_prepare_failed");
    duckdb_disconnect(&conn);
    duckdb_close(&db);
    return -1;
  }
  duckdb_bind_varchar(stmt, 1, workspace_id);
  duckdb_bind_varchar(stmt, 2, record_class);
  duckdb_bind_varchar(stmt, 3, record_key);
  duckdb_bind_varchar(stmt, 4, record_json);
  st = duckdb_execute_prepared(stmt, NULL);
  duckdb_destroy_prepare(&stmt);
  duckdb_disconnect(&conn);
  duckdb_close(&db);
  if (st != DuckDBSuccess) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_insert_failed");
    return -1;
  }
  return 0;
#else
  (void)workspace_id; (void)record_class; (void)record_key; (void)record_json;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_not_available");
  return 1;
#endif
}

int yai_data_duckdb_count(const char *workspace_id,
                          const char *record_class,
                          size_t *out_count,
                          char *err,
                          size_t err_cap)
{
#if defined(YAI_HAVE_DUCKDB)
  duckdb_database db = NULL;
  duckdb_connection conn = NULL;
  duckdb_prepared_statement stmt = NULL;
  duckdb_result result;
  duckdb_state st;
  char path[800];
  if (err && err_cap > 0) err[0] = '\0';
  if (out_count) *out_count = 0;
  if (!workspace_id || !record_class || !out_count) return -1;
  if (yai_data_duckdb_path(workspace_id, path, sizeof(path), err, err_cap) != 0) return -1;
  if (duckdb_open(path, &db) != DuckDBSuccess) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_open_failed");
    return -1;
  }
  if (duckdb_connect(db, &conn) != DuckDBSuccess) {
    duckdb_close(&db);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_connect_failed");
    return -1;
  }
  (void)duckdb_query(conn,
                     "CREATE TABLE IF NOT EXISTS yai_records("
                     "workspace_id VARCHAR, record_class VARCHAR, record_key VARCHAR, payload JSON, created_at TIMESTAMP DEFAULT current_timestamp)",
                     NULL);
  st = duckdb_prepare(conn,
                      "SELECT COUNT(*) FROM yai_records WHERE workspace_id=? AND record_class=?",
                      &stmt);
  if (st != DuckDBSuccess) {
    duckdb_disconnect(&conn);
    duckdb_close(&db);
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_prepare_failed");
    return -1;
  }
  duckdb_bind_varchar(stmt, 1, workspace_id);
  duckdb_bind_varchar(stmt, 2, record_class);
  st = duckdb_execute_prepared(stmt, &result);
  duckdb_destroy_prepare(&stmt);
  if (st == DuckDBSuccess && duckdb_row_count(&result) > 0) {
    *out_count = (size_t)duckdb_value_int64(&result, 0, 0);
  }
  duckdb_destroy_result(&result);
  duckdb_disconnect(&conn);
  duckdb_close(&db);
  if (st != DuckDBSuccess) {
    if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_query_failed");
    return -1;
  }
  return 0;
#else
  (void)workspace_id; (void)record_class; (void)out_count;
  if (err && err_cap > 0) snprintf(err, err_cap, "%s", "duckdb_not_available");
  return -1;
#endif
}

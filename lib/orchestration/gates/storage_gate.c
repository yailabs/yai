#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sqlite3.h>
#include <sys/stat.h>
#include <unistd.h>

#include <yai/orchestration/storage_gate.h>
#include <yai/orchestration/engine_bridge.h>
#include "cJSON.h" 

// Registro delle connessioni per il Multi-tenancy (ADR-001)
typedef struct {
    char ws_id[64];
    sqlite3* db;
    bool active;
} yai_db_conn_t;

#define MAX_OPEN_DBS 16
static yai_db_conn_t db_registry[MAX_OPEN_DBS];
static int g_storage_initialized = 0;

// --- Inizializzazione ---
void yai_storage_init(void) {
    for (int i = 0; i < MAX_OPEN_DBS; i++) {
        memset(&db_registry[i], 0, sizeof(yai_db_conn_t));
        db_registry[i].active = false;
    }
    g_storage_initialized = 1;
}

int yai_exec_storage_gate_ready(void) {
    if (!g_storage_initialized) {
        yai_storage_init();
    }
    return g_storage_initialized ? 1 : 0;
}

// Helper per creare risposte JSON
static char* json_resp(const char* status, const char* code, const char* ws_id) {
    cJSON *root = cJSON_CreateObject();
    cJSON_AddStringToObject(root, "v", "1");
    cJSON_AddStringToObject(root, "status", status);
    if (code) cJSON_AddStringToObject(root, "code", code);
    if (ws_id) cJSON_AddStringToObject(root, "ws_id", ws_id);
    char *out = cJSON_PrintUnformatted(root);
    cJSON_Delete(root);
    return out;
}

// Risolutore dinamico di Database (Lazy Loading)
static sqlite3* get_db_for_ws(const char* ws_id) {
    if (!ws_id || ws_id[0] == '\0') return NULL;

    for (int i = 0; i < MAX_OPEN_DBS; i++) {
        if (db_registry[i].active && strcmp(db_registry[i].ws_id, ws_id) == 0) {
            return db_registry[i].db;
        }
    }

    char db_path[512];
    const char* home = getenv("HOME");
    
    char dir_path[512];
    snprintf(dir_path, sizeof(dir_path), "%s/.yai/run/%s", home, ws_id);
    mkdir(dir_path, 0755);

    snprintf(db_path, sizeof(db_path), "%s/semantic.sqlite", dir_path);

    sqlite3* db = NULL;
    if (sqlite3_open(db_path, &db) != SQLITE_OK) {
        return NULL;
    }

    sqlite3_exec(db, "PRAGMA journal_mode=WAL;", NULL, NULL, NULL);
    sqlite3_exec(db, "PRAGMA synchronous=NORMAL;", NULL, NULL, NULL);

    const char* schema = 
        "CREATE TABLE IF NOT EXISTS nodes ("
        "  id TEXT PRIMARY KEY NOT NULL,"
        "  kind TEXT NOT NULL,"
        "  meta TEXT DEFAULT '{}',"
        "  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP"
        ");"
        "CREATE TABLE IF NOT EXISTS edges ("
        "  source TEXT NOT NULL,"
        "  target TEXT NOT NULL,"
        "  relation TEXT NOT NULL,"
        "  PRIMARY KEY (source, target, relation)"
        ");";
    
    if (sqlite3_exec(db, schema, NULL, NULL, NULL) != SQLITE_OK) {
        sqlite3_close(db);
        return NULL;
    }

    for (int i = 0; i < MAX_OPEN_DBS; i++) {
        if (!db_registry[i].active) {
            strncpy(db_registry[i].ws_id, ws_id, 63);
            db_registry[i].db = db;
            db_registry[i].active = true;
            return db;
        }
    }
    return db;
}

char* yai_storage_handle_rpc(const char* ws_id, const char* method, const char* params_json) {
    sqlite3* db = get_db_for_ws(ws_id);
    if (!db) return json_resp("error", "ERR_STORAGE_UNAVAILABLE", ws_id);

    cJSON *params = cJSON_Parse(params_json);
    if (!params) return json_resp("error", "ERR_INVALID_JSON", ws_id);

    char* response = NULL;

    if (strcmp(method, "put_node") == 0) {
        cJSON *id = cJSON_GetObjectItem(params, "id");
        cJSON *kind = cJSON_GetObjectItem(params, "kind");
        cJSON *meta = cJSON_GetObjectItem(params, "meta");

        if (!id || !kind) {
            response = json_resp("error", "ERR_MISSING_FIELDS", ws_id);
        } else {
            const char* sql = "INSERT OR REPLACE INTO nodes (id, kind, meta) VALUES (?, ?, ?);";
            sqlite3_stmt* stmt;
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) == SQLITE_OK) {
                char* meta_str = cJSON_PrintUnformatted(meta ? meta : cJSON_CreateObject());
                sqlite3_bind_text(stmt, 1, id->valuestring, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 2, kind->valuestring, -1, SQLITE_STATIC);
                sqlite3_bind_text(stmt, 3, meta_str, -1, SQLITE_TRANSIENT);

                if (sqlite3_step(stmt) == SQLITE_DONE) {
                    response = json_resp("ok", NULL, ws_id);
                } else {
                    response = json_resp("error", "ERR_SQL_EXEC", ws_id);
                }
                free(meta_str);
                sqlite3_finalize(stmt);
            }
        }
    } 
    else if (strcmp(method, "get_node") == 0) {
        cJSON *id = cJSON_GetObjectItem(params, "id");
        if (!id) {
            response = json_resp("error", "ERR_MISSING_ID", ws_id);
        } else {
            const char* sql = "SELECT id, kind, meta FROM nodes WHERE id = ?;";
            sqlite3_stmt* stmt;
            if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) == SQLITE_OK) {
                sqlite3_bind_text(stmt, 1, id->valuestring, -1, SQLITE_STATIC);
                if (sqlite3_step(stmt) == SQLITE_ROW) {
                    cJSON *res = cJSON_CreateObject();
                    cJSON_AddStringToObject(res, "status", "ok");
                    cJSON_AddStringToObject(res, "id", (const char*)sqlite3_column_text(stmt, 0));
                    cJSON_AddStringToObject(res, "kind", (const char*)sqlite3_column_text(stmt, 1));
                    cJSON_AddItemToObject(res, "meta", cJSON_Parse((const char*)sqlite3_column_text(stmt, 2)));
                    response = cJSON_PrintUnformatted(res);
                    cJSON_Delete(res);
                } else {
                    response = json_resp("error", "ERR_NODE_NOT_FOUND", ws_id);
                }
                sqlite3_finalize(stmt);
            }
        }
    } else {
        response = json_resp("error", "ERR_METHOD_NOT_SUPPORTED", ws_id);
    }

    cJSON_Delete(params);
    return response;
}

void yai_storage_shutdown(void) {
    for (int i = 0; i < MAX_OPEN_DBS; i++) {
        if (db_registry[i].active) {
            sqlite3_close(db_registry[i].db);
            db_registry[i].db = NULL;
            db_registry[i].active = false;
        }
    }
    g_storage_initialized = 0;
}

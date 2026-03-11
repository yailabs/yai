#define _POSIX_C_SOURCE 200809L

#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

#include <cJSON.h>

#include <protocol.h>
#include <transport.h>
#include <yai_protocol_ids.h>

#include <yai/daemon/local_runtime.h>
#include <yai/daemon/source_ids.h>
#include <yai/daemon/source_plane_model.h>
#include <yai/daemon/runtime.h>

#include "internal.h"

typedef struct yai_daemon_unit {
  char unit_id[128];
  char workspace_id[64];
  char source_binding_id[96];
  char source_node_id[96];
  char locator[512];
  char asset_type[64];
  char fingerprint[96];
  char idempotency_key[96];
  int64_t observed_at_epoch;
  int attempts;
  int64_t next_attempt_epoch;
  char status[32];
  char last_error[128];
} yai_daemon_unit_t;

static const yai_daemon_config_t *g_cfg = NULL;
static const yai_daemon_paths_t *g_paths = NULL;
static char g_instance_id[96];
static uint32_t g_local_seq = 0;

static int64_t now_epoch(void)
{
  return (int64_t)time(NULL);
}

static uint64_t fnv1a64(const char *s)
{
  uint64_t h = 1469598103934665603ULL;
  size_t i = 0;
  if (!s) return h;
  for (i = 0; s[i]; ++i) {
    h ^= (uint64_t)(unsigned char)s[i];
    h *= 1099511628211ULL;
  }
  return h;
}

static int compact_id(char *out, size_t out_cap, const char *prefix, const char *a, const char *b)
{
  uint64_t h = fnv1a64(a);
  h ^= fnv1a64(b);
  g_local_seq += 1U;
  if (!out || out_cap == 0 || !prefix || !prefix[0]) return -1;
  if (snprintf(out, out_cap, "%s-%llx-%u", prefix, (unsigned long long)h, g_local_seq) >= (int)out_cap) return -1;
  return 0;
}

static int write_all(int fd, const void *buf, size_t len)
{
  const char *p = (const char *)buf;
  size_t off = 0;
  while (off < len) {
    ssize_t n = write(fd, p + off, len - off);
    if (n < 0) {
      if (errno == EINTR) continue;
      return -1;
    }
    off += (size_t)n;
  }
  return 0;
}

static int read_all(int fd, void *buf, size_t len)
{
  char *p = (char *)buf;
  size_t off = 0;
  while (off < len) {
    ssize_t n = read(fd, p + off, len - off);
    if (n <= 0) {
      if (n < 0 && errno == EINTR) continue;
      return -1;
    }
    off += (size_t)n;
  }
  return 0;
}

static int parse_owner_socket(const char *owner_ref, char *out, size_t out_cap)
{
  const char *p = owner_ref;
  if (!out || out_cap == 0) return -1;
  out[0] = '\0';
  if (!owner_ref || !owner_ref[0]) return -1;

  if (strncmp(owner_ref, "unix://", 7) == 0) p = owner_ref + 7;
  else if (strncmp(owner_ref, "uds://", 6) == 0) p = owner_ref + 6;
  else if (strncmp(owner_ref, "unix:", 5) == 0) p = owner_ref + 5;
  if (!p[0] || p[0] != '/') return -1;
  if (snprintf(out, out_cap, "%s", p) >= (int)out_cap) return -1;
  return 0;
}

static int connect_socket(const char *socket_path)
{
  struct sockaddr_un addr;
  socklen_t len = 0;
  int fd = -1;
  if (!socket_path || !socket_path[0]) return -1;
  fd = socket(AF_UNIX, SOCK_STREAM, 0);
  if (fd < 0) return -1;
  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  if (snprintf(addr.sun_path, sizeof(addr.sun_path), "%s", socket_path) >= (int)sizeof(addr.sun_path)) {
    close(fd);
    return -1;
  }
  len = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + strlen(addr.sun_path));
  if (connect(fd, (struct sockaddr *)&addr, len) != 0) {
    close(fd);
    return -1;
  }
  return fd;
}

static int rpc_handshake(int fd, const char *ws_id)
{
  yai_rpc_envelope_t env;
  yai_rpc_envelope_t reply;
  yai_handshake_req_t req;
  yai_handshake_ack_t ack;
  (void)ws_id;
  memset(&env, 0, sizeof(env));
  memset(&req, 0, sizeof(req));
  env.magic = YAI_FRAME_MAGIC;
  env.version = YAI_PROTOCOL_IDS_VERSION;
  env.command_id = YAI_CMD_HANDSHAKE;
  env.role = 2;
  env.arming = 1;
  env.payload_len = (uint32_t)sizeof(req);
  snprintf(env.ws_id, sizeof(env.ws_id), "%s", ws_id && ws_id[0] ? ws_id : "system");
  snprintf(env.trace_id, sizeof(env.trace_id), "%s", "yd5-hs");

  req.client_version = YAI_PROTOCOL_IDS_VERSION;
  req.capabilities_requested = 0;
  snprintf(req.client_name, sizeof(req.client_name), "%s", "yai-daemon");

  if (write_all(fd, &env, sizeof(env)) != 0) return -1;
  if (write_all(fd, &req, sizeof(req)) != 0) return -1;
  if (read_all(fd, &reply, sizeof(reply)) != 0) return -1;
  if (reply.magic != YAI_FRAME_MAGIC || reply.command_id != YAI_CMD_HANDSHAKE) return -1;
  if (reply.payload_len != sizeof(ack)) return -1;
  if (read_all(fd, &ack, sizeof(ack)) != 0) return -1;
  if (ack.status != YAI_PROTO_STATE_READY) return -1;
  return 0;
}

static int rpc_control_call(const char *socket_path,
                            const char *ws_id,
                            const char *payload,
                            char *reply_out,
                            size_t reply_cap)
{
  int fd = -1;
  yai_rpc_envelope_t env;
  yai_rpc_envelope_t rep_env;
  uint32_t plen = 0;
  if (!socket_path || !payload || !reply_out || reply_cap == 0) return -1;
  reply_out[0] = '\0';

  fd = connect_socket(socket_path);
  if (fd < 0) return -2;
  if (rpc_handshake(fd, ws_id) != 0) {
    close(fd);
    return -3;
  }

  memset(&env, 0, sizeof(env));
  env.magic = YAI_FRAME_MAGIC;
  env.version = YAI_PROTOCOL_IDS_VERSION;
  env.command_id = YAI_CMD_CONTROL_CALL;
  env.role = 2;
  env.arming = 1;
  plen = (uint32_t)strlen(payload);
  if (plen > YAI_MAX_PAYLOAD) {
    close(fd);
    return -4;
  }
  env.payload_len = plen;
  snprintf(env.ws_id, sizeof(env.ws_id), "%s", ws_id && ws_id[0] ? ws_id : "system");
  snprintf(env.trace_id, sizeof(env.trace_id), "%s", "yd5-cc");

  if (write_all(fd, &env, sizeof(env)) != 0 || write_all(fd, payload, plen) != 0) {
    close(fd);
    return -5;
  }
  if (read_all(fd, &rep_env, sizeof(rep_env)) != 0) {
    close(fd);
    return -6;
  }
  if (rep_env.magic != YAI_FRAME_MAGIC || rep_env.command_id != YAI_CMD_CONTROL_CALL) {
    close(fd);
    return -7;
  }
  if (rep_env.payload_len >= reply_cap) {
    close(fd);
    return -8;
  }
  if (rep_env.payload_len > 0) {
    if (read_all(fd, reply_out, rep_env.payload_len) != 0) {
      close(fd);
      return -9;
    }
    reply_out[rep_env.payload_len] = '\0';
  } else {
    reply_out[0] = '\0';
  }
  close(fd);
  return 0;
}

static int json_reply_ok(const char *reply)
{
  return reply && strstr(reply, "\"status\":\"ok\"") != NULL;
}

static int json_extract_string(const char *json, const char *key, char *out, size_t out_cap)
{
  cJSON *root = NULL;
  cJSON *it = NULL;
  if (!json || !key || !out || out_cap == 0) return -1;
  out[0] = '\0';
  root = cJSON_Parse(json);
  if (!root) return -1;
  it = cJSON_GetObjectItemCaseSensitive(root, key);
  if (cJSON_IsString(it) && it->valuestring) {
    snprintf(out, out_cap, "%s", it->valuestring);
    cJSON_Delete(root);
    return 0;
  }
  cJSON_Delete(root);
  return -1;
}

static int ensure_dir(const char *path)
{
  return yai_daemon_mkdir_recursive(path);
}

static int write_json_file(const char *path, cJSON *obj)
{
  char *txt = NULL;
  int rc = -1;
  if (!path || !obj) return -1;
  txt = cJSON_PrintUnformatted(obj);
  if (!txt) return -1;
  rc = yai_daemon_write_file(path, txt);
  free(txt);
  return rc;
}

static int build_paths(yai_daemon_local_runtime_t *local)
{
  if (!local || !g_paths) return -1;
  if (snprintf(local->queue_dir, sizeof(local->queue_dir), "%s/queue", g_paths->spool_dir) >= (int)sizeof(local->queue_dir)) return -1;
  if (snprintf(local->delivered_dir, sizeof(local->delivered_dir), "%s/delivered", g_paths->spool_dir) >= (int)sizeof(local->delivered_dir)) return -1;
  if (snprintf(local->failed_dir, sizeof(local->failed_dir), "%s/failed", g_paths->spool_dir) >= (int)sizeof(local->failed_dir)) return -1;
  if (snprintf(local->observed_index_file, sizeof(local->observed_index_file), "%s/observed-assets.v1.tsv", g_paths->state_dir) >= (int)sizeof(local->observed_index_file)) return -1;
  if (snprintf(local->bindings_state_file, sizeof(local->bindings_state_file), "%s/bindings.v1.json", g_paths->state_dir) >= (int)sizeof(local->bindings_state_file)) return -1;
  if (ensure_dir(local->queue_dir) != 0 || ensure_dir(local->delivered_dir) != 0 || ensure_dir(local->failed_dir) != 0) return -1;
  return 0;
}

static int refresh_spool_counts(yai_daemon_local_runtime_t *local)
{
  DIR *d = NULL;
  struct dirent *de;
  if (!local) return -1;
  local->spool_queued = 0;
  local->spool_delivered = 0;
  local->spool_failed = 0;

  d = opendir(local->queue_dir);
  if (d) {
    while ((de = readdir(d)) != NULL) {
      if (de->d_name[0] == '.') continue;
      local->spool_queued++;
    }
    closedir(d);
  }
  d = opendir(local->delivered_dir);
  if (d) {
    while ((de = readdir(d)) != NULL) {
      if (de->d_name[0] == '.') continue;
      local->spool_delivered++;
    }
    closedir(d);
  }
  d = opendir(local->failed_dir);
  if (d) {
    while ((de = readdir(d)) != NULL) {
      if (de->d_name[0] == '.') continue;
      local->spool_failed++;
    }
    closedir(d);
  }
  return 0;
}

static int persist_observed_index(const yai_daemon_local_runtime_t *local)
{
  FILE *f = NULL;
  size_t i = 0;
  if (!local || !local->observed_index_file[0]) return -1;
  f = fopen(local->observed_index_file, "w");
  if (!f) return -1;
  for (i = 0; i < local->observed_count; ++i) {
    fprintf(f, "%s\t%s\n", local->observed[i].key, local->observed[i].fingerprint);
  }
  fclose(f);
  return 0;
}

static int load_observed_index(yai_daemon_local_runtime_t *local)
{
  FILE *f = NULL;
  char line[768];
  if (!local || !local->observed_index_file[0]) return -1;
  local->observed_count = 0;
  f = fopen(local->observed_index_file, "r");
  if (!f) return 0;
  while (fgets(line, sizeof(line), f) && local->observed_count < YAI_DAEMON_MAX_OBSERVED) {
    char *tab = strchr(line, '\t');
    char *nl = NULL;
    if (!tab) continue;
    *tab = '\0';
    tab++;
    nl = strchr(tab, '\n');
    if (nl) *nl = '\0';
    snprintf(local->observed[local->observed_count].key, sizeof(local->observed[local->observed_count].key), "%s", line);
    snprintf(local->observed[local->observed_count].fingerprint, sizeof(local->observed[local->observed_count].fingerprint), "%s", tab);
    local->observed_count++;
  }
  fclose(f);
  return 0;
}

static int observed_find(const yai_daemon_local_runtime_t *local, const char *key)
{
  size_t i;
  if (!local || !key || !key[0]) return -1;
  for (i = 0; i < local->observed_count; ++i) {
    if (strcmp(local->observed[i].key, key) == 0) return (int)i;
  }
  return -1;
}

static int observed_upsert(yai_daemon_local_runtime_t *local, const char *key, const char *fingerprint)
{
  int idx = -1;
  if (!local || !key || !fingerprint) return -1;
  idx = observed_find(local, key);
  if (idx >= 0) {
    snprintf(local->observed[(size_t)idx].fingerprint, sizeof(local->observed[(size_t)idx].fingerprint), "%s", fingerprint);
    return 0;
  }
  if (local->observed_count >= YAI_DAEMON_MAX_OBSERVED) return -1;
  snprintf(local->observed[local->observed_count].key, sizeof(local->observed[local->observed_count].key), "%s", key);
  snprintf(local->observed[local->observed_count].fingerprint, sizeof(local->observed[local->observed_count].fingerprint), "%s", fingerprint);
  local->observed_count++;
  return 0;
}

static int load_bindings_manifest(yai_daemon_local_runtime_t *local)
{
  FILE *f = NULL;
  char *buf = NULL;
  long sz = 0;
  size_t rd = 0;
  cJSON *root = NULL;
  cJSON *arr = NULL;
  cJSON *it = NULL;
  if (!local || !g_cfg) return -1;

  local->binding_count = 0;
  f = fopen(g_cfg->bindings_manifest, "r");
  if (!f) return 0;
  if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return -1; }
  sz = ftell(f);
  if (sz <= 0 || sz > (1024 * 1024)) { fclose(f); return -1; }
  if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return -1; }
  buf = (char *)calloc((size_t)sz + 1, 1);
  if (!buf) { fclose(f); return -1; }
  rd = fread(buf, 1, (size_t)sz, f);
  fclose(f);
  buf[rd] = '\0';

  root = cJSON_Parse(buf);
  free(buf);
  if (!root) return -1;
  arr = cJSON_GetObjectItemCaseSensitive(root, "bindings");
  if (!cJSON_IsArray(arr)) {
    cJSON_Delete(root);
    return 0;
  }
  cJSON_ArrayForEach(it, arr) {
    cJSON *ws = NULL;
    cJSON *rp = NULL;
    cJSON *id = NULL;
    cJSON *sc = NULL;
    cJSON *at = NULL;
    cJSON *en = NULL;
    yai_daemon_binding_rt_t *b = NULL;
    if (local->binding_count >= YAI_DAEMON_MAX_BINDINGS) break;
    ws = cJSON_GetObjectItemCaseSensitive(it, "workspace_id");
    rp = cJSON_GetObjectItemCaseSensitive(it, "root_path");
    if (!cJSON_IsString(ws) || !ws->valuestring || !ws->valuestring[0]) continue;
    if (!cJSON_IsString(rp) || !rp->valuestring || !rp->valuestring[0]) continue;
    b = &local->bindings[local->binding_count];
    memset(b, 0, sizeof(*b));
    snprintf(b->workspace_id, sizeof(b->workspace_id), "%s", ws->valuestring);
    snprintf(b->root_path, sizeof(b->root_path), "%s", rp->valuestring);
    id = cJSON_GetObjectItemCaseSensitive(it, "binding_id");
    if (cJSON_IsString(id) && id->valuestring && id->valuestring[0]) {
      snprintf(b->binding_id, sizeof(b->binding_id), "%s", id->valuestring);
    } else {
      (void)yai_source_id_binding(b->binding_id, sizeof(b->binding_id), local->source_node_id, b->workspace_id);
    }
    sc = cJSON_GetObjectItemCaseSensitive(it, "scope");
    snprintf(b->scope, sizeof(b->scope), "%s",
             (cJSON_IsString(sc) && sc->valuestring && sc->valuestring[0]) ? sc->valuestring : "workspace");
    at = cJSON_GetObjectItemCaseSensitive(it, "asset_type");
    snprintf(b->asset_type, sizeof(b->asset_type), "%s",
             (cJSON_IsString(at) && at->valuestring && at->valuestring[0]) ? at->valuestring : "file");
    en = cJSON_GetObjectItemCaseSensitive(it, "enabled");
    b->enabled = cJSON_IsBool(en) ? cJSON_IsTrue(en) : 1;
    snprintf(b->status, sizeof(b->status), "%s", b->enabled ? YAI_DAEMON_BINDING_STATUS_CONFIGURED : YAI_DAEMON_BINDING_STATUS_INVALID);
    local->binding_count++;
  }
  cJSON_Delete(root);
  return 0;
}

static int write_bindings_state(const yai_daemon_local_runtime_t *local)
{
  cJSON *root = NULL;
  cJSON *arr = NULL;
  size_t i = 0;
  int rc = -1;
  if (!local) return -1;
  root = cJSON_CreateObject();
  if (!root) return -1;
  cJSON_AddStringToObject(root, "type", "yai.daemon.bindings.state.v1");
  cJSON_AddStringToObject(root, "instance_id", g_instance_id);
  arr = cJSON_AddArrayToObject(root, "bindings");
  if (!arr) { cJSON_Delete(root); return -1; }
  for (i = 0; i < local->binding_count; ++i) {
    cJSON *o = cJSON_CreateObject();
    if (!o) continue;
    cJSON_AddStringToObject(o, "binding_id", local->bindings[i].binding_id);
    cJSON_AddStringToObject(o, "workspace_id", local->bindings[i].workspace_id);
    cJSON_AddStringToObject(o, "root_path", local->bindings[i].root_path);
    cJSON_AddStringToObject(o, "status", local->bindings[i].status);
    cJSON_AddBoolToObject(o, "enabled", local->bindings[i].enabled ? 1 : 0);
    cJSON_AddItemToArray(arr, o);
  }
  rc = write_json_file(local->bindings_state_file, root);
  cJSON_Delete(root);
  return rc;
}

static void file_fingerprint(const struct stat *st, char *out, size_t out_cap)
{
  if (!st || !out || out_cap == 0) return;
  snprintf(out, out_cap, "size:%lld-mtime:%lld", (long long)st->st_size, (long long)st->st_mtime);
}

static int create_unit_from_file(yai_daemon_local_runtime_t *local,
                                 const yai_daemon_binding_rt_t *binding,
                                 const char *file_name,
                                 const struct stat *st)
{
  char locator[512];
  char key[512];
  char fp[96];
  int idx = -1;
  char queue_file[512];
  cJSON *unit = NULL;
  int rc = -1;
  int64_t t = now_epoch();
  char source_asset_id[128];
  char source_event_id[128];

  if (!local || !binding || !file_name || !st) return -1;
  if (snprintf(locator, sizeof(locator), "file://%s/%s", binding->root_path, file_name) >= (int)sizeof(locator)) return -1;
  if (snprintf(key, sizeof(key), "%s|%s", binding->binding_id, locator) >= (int)sizeof(key)) return -1;
  file_fingerprint(st, fp, sizeof(fp));
  idx = observed_find(local, key);
  if (idx >= 0 && strcmp(local->observed[(size_t)idx].fingerprint, fp) == 0) return 0;
  if (observed_upsert(local, key, fp) != 0) return -1;

  unit = cJSON_CreateObject();
  if (!unit) return -1;
  if (compact_id(source_asset_id, sizeof(source_asset_id), "sa", binding->binding_id, locator) != 0 ||
      compact_id(source_event_id, sizeof(source_event_id), "se", binding->binding_id, fp) != 0) {
    cJSON_Delete(unit);
    return -1;
  }

  cJSON_AddStringToObject(unit, "type", "yai.daemon.acquisition.unit.v1");
  cJSON_AddStringToObject(unit, "unit_id", source_event_id);
  cJSON_AddStringToObject(unit, "workspace_id", binding->workspace_id);
  cJSON_AddStringToObject(unit, "source_binding_id", binding->binding_id);
  cJSON_AddStringToObject(unit, "source_node_id", local->source_node_id);
  cJSON_AddStringToObject(unit, "locator", locator);
  cJSON_AddStringToObject(unit, "asset_type", binding->asset_type);
  cJSON_AddStringToObject(unit, "fingerprint", fp);
  cJSON_AddStringToObject(unit, "source_asset_id", source_asset_id);
  cJSON_AddStringToObject(unit, "source_acquisition_event_id", source_event_id);
  cJSON_AddStringToObject(unit, "idempotency_key", source_event_id);
  cJSON_AddNumberToObject(unit, "observed_at_epoch", (double)t);
  cJSON_AddNumberToObject(unit, "attempts", 0);
  cJSON_AddNumberToObject(unit, "next_attempt_epoch", (double)t);
  cJSON_AddStringToObject(unit, "status", YAI_DAEMON_UNIT_STATUS_QUEUED);
  cJSON_AddStringToObject(unit, "last_error", "none");

  if (snprintf(queue_file, sizeof(queue_file), "%s/%s.json", local->queue_dir, source_event_id) >= (int)sizeof(queue_file)) {
    cJSON_Delete(unit);
    return -1;
  }
  rc = write_json_file(queue_file, unit);
  cJSON_Delete(unit);
  if (rc == 0) local->scan_discovered++;
  return rc;
}

static int scan_binding(yai_daemon_local_runtime_t *local, const yai_daemon_binding_rt_t *binding)
{
  DIR *d = NULL;
  struct dirent *de = NULL;
  if (!local || !binding || !binding->enabled) return -1;
  d = opendir(binding->root_path);
  if (!d) return -1;
  while ((de = readdir(d)) != NULL) {
    char path[1024];
    struct stat st;
    if (de->d_name[0] == '.') continue;
    if (snprintf(path, sizeof(path), "%s/%s", binding->root_path, de->d_name) >= (int)sizeof(path)) continue;
    if (stat(path, &st) != 0) continue;
    if (!S_ISREG(st.st_mode)) continue;
    (void)create_unit_from_file(local, binding, de->d_name, &st);
  }
  closedir(d);
  return 0;
}

static int ensure_owner_registered(yai_daemon_local_runtime_t *local, const char *workspace_id)
{
  char payload[1024];
  char reply[4096];
  int rc = 0;
  if (!local || !workspace_id || !workspace_id[0] || !local->owner_socket[0]) return -1;
  if (local->owner_registered) return 0;

  snprintf(payload,
           sizeof(payload),
           "{\"type\":\"yai.control.call.v1\",\"command_id\":\"yai.source.enroll\",\"target_plane\":\"runtime\",\"workspace_id\":\"%s\",\"source_label\":\"%s\",\"owner_ref\":\"%s\"}",
           workspace_id,
           g_cfg->source_label,
           g_cfg->owner_ref);
  rc = rpc_control_call(local->owner_socket, workspace_id, payload, reply, sizeof(reply));
  if (rc != 0 || !json_reply_ok(reply)) return -1;
  (void)json_extract_string(reply, "source_node_id", local->source_node_id, sizeof(local->source_node_id));
  (void)json_extract_string(reply, "daemon_instance_id", local->daemon_instance_id, sizeof(local->daemon_instance_id));
  (void)json_extract_string(reply, "owner_link_id", local->owner_link_id, sizeof(local->owner_link_id));
  (void)json_extract_string(reply, "source_enrollment_grant_id", local->source_enrollment_grant_id, sizeof(local->source_enrollment_grant_id));
  (void)json_extract_string(reply, "owner_trust_artifact_id", local->owner_trust_artifact_id, sizeof(local->owner_trust_artifact_id));
  (void)json_extract_string(reply, "owner_trust_artifact_token", local->owner_trust_artifact_token, sizeof(local->owner_trust_artifact_token));
  if (!local->owner_trust_artifact_token[0] || strcmp(local->owner_trust_artifact_token, "pending") == 0) return -1;
  local->owner_registered = 1;
  local->owner_connected = 1;
  local->last_owner_contact_epoch = now_epoch();
  return 0;
}

static int ensure_binding_attached(yai_daemon_local_runtime_t *local, yai_daemon_binding_rt_t *binding)
{
  char payload[1024];
  char reply[4096];
  int rc = 0;
  if (!local || !binding || !local->owner_socket[0]) return -1;
  if (strcmp(binding->status, YAI_DAEMON_BINDING_STATUS_ACTIVE) == 0) return 0;
  if (ensure_owner_registered(local, binding->workspace_id) != 0) return -1;

  snprintf(payload,
           sizeof(payload),
           "{\"type\":\"yai.control.call.v1\",\"command_id\":\"yai.source.attach\",\"target_plane\":\"runtime\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"owner_trust_artifact_id\":\"%s\",\"owner_trust_artifact_token\":\"%s\",\"binding_scope\":\"%s\"}",
           binding->workspace_id,
           local->source_node_id,
           local->owner_trust_artifact_id,
           local->owner_trust_artifact_token,
           binding->scope[0] ? binding->scope : "workspace");
  rc = rpc_control_call(local->owner_socket, binding->workspace_id, payload, reply, sizeof(reply));
  if (rc != 0 || !json_reply_ok(reply)) return -1;
  (void)json_extract_string(reply, "source_binding_id", binding->binding_id, sizeof(binding->binding_id));
  snprintf(binding->status, sizeof(binding->status), "%s", YAI_DAEMON_BINDING_STATUS_ACTIVE);
  local->owner_connected = 1;
  local->last_owner_contact_epoch = now_epoch();
  return 0;
}

static int parse_unit_file(const char *path, yai_daemon_unit_t *unit)
{
  FILE *f = NULL;
  long sz = 0;
  size_t rd = 0;
  char *buf = NULL;
  cJSON *root = NULL;
  if (!path || !unit) return -1;
  memset(unit, 0, sizeof(*unit));
  f = fopen(path, "r");
  if (!f) return -1;
  if (fseek(f, 0, SEEK_END) != 0) { fclose(f); return -1; }
  sz = ftell(f);
  if (sz <= 0 || sz > (1024 * 1024)) { fclose(f); return -1; }
  if (fseek(f, 0, SEEK_SET) != 0) { fclose(f); return -1; }
  buf = (char *)calloc((size_t)sz + 1, 1);
  if (!buf) { fclose(f); return -1; }
  rd = fread(buf, 1, (size_t)sz, f);
  fclose(f);
  buf[rd] = '\0';
  root = cJSON_Parse(buf);
  free(buf);
  if (!root) return -1;
#define READ_S(KEY, DST) do { cJSON *x = cJSON_GetObjectItemCaseSensitive(root, KEY); if (cJSON_IsString(x) && x->valuestring) snprintf(DST, sizeof(DST), "%s", x->valuestring); } while(0)
  READ_S("unit_id", unit->unit_id);
  READ_S("workspace_id", unit->workspace_id);
  READ_S("source_binding_id", unit->source_binding_id);
  READ_S("source_node_id", unit->source_node_id);
  READ_S("locator", unit->locator);
  READ_S("asset_type", unit->asset_type);
  READ_S("fingerprint", unit->fingerprint);
  READ_S("idempotency_key", unit->idempotency_key);
  READ_S("status", unit->status);
  READ_S("last_error", unit->last_error);
  {
    cJSON *n = cJSON_GetObjectItemCaseSensitive(root, "observed_at_epoch");
    if (cJSON_IsNumber(n)) unit->observed_at_epoch = (int64_t)n->valuedouble;
    n = cJSON_GetObjectItemCaseSensitive(root, "attempts");
    if (cJSON_IsNumber(n)) unit->attempts = n->valueint;
    n = cJSON_GetObjectItemCaseSensitive(root, "next_attempt_epoch");
    if (cJSON_IsNumber(n)) unit->next_attempt_epoch = (int64_t)n->valuedouble;
  }
#undef READ_S
  cJSON_Delete(root);
  return 0;
}

static int write_unit_file(const char *path, const yai_daemon_unit_t *unit)
{
  cJSON *o = NULL;
  int rc = -1;
  if (!path || !unit) return -1;
  o = cJSON_CreateObject();
  if (!o) return -1;
  cJSON_AddStringToObject(o, "type", "yai.daemon.acquisition.unit.v1");
  cJSON_AddStringToObject(o, "unit_id", unit->unit_id);
  cJSON_AddStringToObject(o, "workspace_id", unit->workspace_id);
  cJSON_AddStringToObject(o, "source_binding_id", unit->source_binding_id);
  cJSON_AddStringToObject(o, "source_node_id", unit->source_node_id);
  cJSON_AddStringToObject(o, "locator", unit->locator);
  cJSON_AddStringToObject(o, "asset_type", unit->asset_type);
  cJSON_AddStringToObject(o, "fingerprint", unit->fingerprint);
  cJSON_AddStringToObject(o, "idempotency_key", unit->idempotency_key);
  cJSON_AddNumberToObject(o, "observed_at_epoch", (double)unit->observed_at_epoch);
  cJSON_AddNumberToObject(o, "attempts", unit->attempts);
  cJSON_AddNumberToObject(o, "next_attempt_epoch", (double)unit->next_attempt_epoch);
  cJSON_AddStringToObject(o, "status", unit->status);
  cJSON_AddStringToObject(o, "last_error", unit->last_error);
  rc = write_json_file(path, o);
  cJSON_Delete(o);
  return rc;
}

static int emit_unit(yai_daemon_local_runtime_t *local, const yai_daemon_unit_t *unit)
{
  char source_asset_id[128];
  char source_event_id[128];
  char source_candidate_id[128];
  char payload[4096];
  char reply[4096];
  int rc = 0;
  if (!local || !unit) return -1;
  if (compact_id(source_asset_id, sizeof(source_asset_id), "sa", unit->source_binding_id, unit->locator) != 0) return -1;
  if (compact_id(source_event_id, sizeof(source_event_id), "se", unit->source_binding_id, unit->fingerprint) != 0) return -1;
  if (compact_id(source_candidate_id, sizeof(source_candidate_id), "sc", source_event_id, "file_observation") != 0) return -1;
  snprintf(payload,
           sizeof(payload),
           "{\"type\":\"yai.control.call.v1\",\"command_id\":\"yai.source.emit\",\"target_plane\":\"runtime\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"owner_trust_artifact_id\":\"%s\",\"owner_trust_artifact_token\":\"%s\",\"idempotency_key\":\"%s\","
           "\"source_assets\":[{\"type\":\"yai.source_asset.v1\",\"source_asset_id\":\"%s\",\"source_binding_id\":\"%s\",\"locator\":\"%s\",\"asset_type\":\"%s\",\"provenance_fingerprint\":\"%s\",\"observation_state\":\"observed\"}],"
           "\"source_acquisition_events\":[{\"type\":\"yai.source_acquisition_event.v1\",\"source_acquisition_event_id\":\"%s\",\"source_node_id\":\"%s\",\"source_binding_id\":\"%s\",\"source_asset_id\":\"%s\",\"event_type\":\"discovered\",\"observed_at_epoch\":%lld,\"idempotency_key\":\"%s\",\"delivery_status\":\"queued\"}],"
           "\"source_evidence_candidates\":[{\"type\":\"yai.source_evidence_candidate.v1\",\"source_evidence_candidate_id\":\"%s\",\"source_acquisition_event_id\":\"%s\",\"candidate_type\":\"file_observation\",\"derived_metadata_ref\":\"meta://daemon/%s\",\"owner_resolution_status\":\"pending\"}]}",
           unit->workspace_id,
           local->source_node_id,
           unit->source_binding_id,
           local->owner_trust_artifact_id,
           local->owner_trust_artifact_token,
           unit->idempotency_key,
           source_asset_id,
           unit->source_binding_id,
           unit->locator,
           unit->asset_type[0] ? unit->asset_type : "file",
           unit->fingerprint,
           source_event_id,
           local->source_node_id,
           unit->source_binding_id,
           source_asset_id,
           (long long)unit->observed_at_epoch,
           unit->idempotency_key,
           source_candidate_id,
           source_event_id,
           unit->idempotency_key);

  rc = rpc_control_call(local->owner_socket, unit->workspace_id, payload, reply, sizeof(reply));
  if (rc != 0 || !json_reply_ok(reply)) return -1;
  local->owner_connected = 1;
  local->last_owner_contact_epoch = now_epoch();
  return 0;
}

static int send_status_update(yai_daemon_local_runtime_t *local, const char *workspace_id)
{
  char payload[1024];
  char reply[4096];
  int rc = 0;
  if (!local || !workspace_id || !workspace_id[0] || !local->owner_socket[0]) return -1;
  if (ensure_owner_registered(local, workspace_id) != 0) return -1;
  snprintf(payload,
           sizeof(payload),
           "{\"type\":\"yai.control.call.v1\",\"command_id\":\"yai.source.status\",\"target_plane\":\"runtime\",\"workspace_id\":\"%s\",\"source_node_id\":\"%s\",\"daemon_instance_id\":\"%s\",\"owner_trust_artifact_id\":\"%s\",\"owner_trust_artifact_token\":\"%s\",\"health\":\"%s\"}",
           workspace_id,
           local->source_node_id,
           local->daemon_instance_id,
           local->owner_trust_artifact_id,
           local->owner_trust_artifact_token,
           local->health_state[0] ? local->health_state : YAI_DAEMON_HEALTH_READY);
  rc = rpc_control_call(local->owner_socket, workspace_id, payload, reply, sizeof(reply));
  if (rc != 0 || !json_reply_ok(reply)) return -1;
  local->owner_connected = 1;
  local->last_owner_contact_epoch = now_epoch();
  return 0;
}

static int process_queue(yai_daemon_local_runtime_t *local)
{
  DIR *d = NULL;
  struct dirent *de = NULL;
  int64_t now = now_epoch();
  if (!local) return -1;
  d = opendir(local->queue_dir);
  if (!d) return -1;
  while ((de = readdir(d)) != NULL) {
    char path[768];
    char done_path[768];
    char fail_path[768];
    yai_daemon_unit_t unit;
    yai_daemon_binding_rt_t *binding = NULL;
    size_t i = 0;
    int emit_rc = -1;
    if (de->d_name[0] == '.') continue;
    if (snprintf(path, sizeof(path), "%s/%s", local->queue_dir, de->d_name) >= (int)sizeof(path)) continue;
    if (parse_unit_file(path, &unit) != 0) continue;
    if (unit.next_attempt_epoch > now) continue;
    for (i = 0; i < local->binding_count; ++i) {
      if (strcmp(local->bindings[i].binding_id, unit.source_binding_id) == 0) { binding = &local->bindings[i]; break; }
    }
    if (!binding) continue;
    if (ensure_binding_attached(local, binding) != 0) {
      local->owner_connected = 0;
      unit.attempts += 1;
      unit.next_attempt_epoch = now + (int64_t)(unit.attempts < 6 ? (1 << unit.attempts) : 60);
      snprintf(unit.status, sizeof(unit.status), "%s", YAI_DAEMON_UNIT_STATUS_RETRY_DUE);
      snprintf(unit.last_error, sizeof(unit.last_error), "%s", "attach_failed");
      (void)write_unit_file(path, &unit);
      local->emit_failures++;
      continue;
    }
    local->emit_attempts++;
    emit_rc = emit_unit(local, &unit);
    if (emit_rc == 0) {
      snprintf(unit.status, sizeof(unit.status), "%s", YAI_DAEMON_UNIT_STATUS_DELIVERED);
      snprintf(unit.last_error, sizeof(unit.last_error), "%s", "none");
      if (snprintf(done_path, sizeof(done_path), "%s/%s", local->delivered_dir, de->d_name) < (int)sizeof(done_path)) {
        (void)write_unit_file(done_path, &unit);
        (void)unlink(path);
      }
      local->emit_success++;
      continue;
    }
    local->owner_connected = 0;
    unit.attempts += 1;
    if (unit.attempts >= 5) {
      snprintf(unit.status, sizeof(unit.status), "%s", YAI_DAEMON_UNIT_STATUS_FAILED);
      snprintf(unit.last_error, sizeof(unit.last_error), "%s", "emit_failed_terminal");
      if (snprintf(fail_path, sizeof(fail_path), "%s/%s", local->failed_dir, de->d_name) < (int)sizeof(fail_path)) {
        (void)write_unit_file(fail_path, &unit);
        (void)unlink(path);
      }
    } else {
      unit.next_attempt_epoch = now + (int64_t)(unit.attempts < 6 ? (1 << unit.attempts) : 60);
      snprintf(unit.status, sizeof(unit.status), "%s", YAI_DAEMON_UNIT_STATUS_RETRY_DUE);
      snprintf(unit.last_error, sizeof(unit.last_error), "%s", "emit_failed_retry");
      (void)write_unit_file(path, &unit);
    }
    local->emit_failures++;
  }
  closedir(d);
  return 0;
}

int yai_daemon_local_runtime_health_json(const yai_daemon_local_runtime_t *local, char *out, size_t out_cap)
{
  if (!local || !out || out_cap == 0) return -1;
  if (snprintf(out,
               out_cap,
               "{"
               "\"type\":\"yai.daemon.health.v2\","
               "\"instance_id\":\"%s\","
               "\"state\":\"%s\","
               "\"bindings_active\":%zu,"
               "\"spool\":{\"queued\":%u,\"retry_due\":%u,\"delivered\":%u,\"failed\":%u},"
               "\"scan_discovered\":%u,"
               "\"emit\":{\"attempts\":%u,\"success\":%u,\"failures\":%u},"
               "\"owner\":{\"connected\":%s,\"registered\":%s,\"last_contact_epoch\":%lld}"
               "}",
               g_instance_id,
               local->health_state[0] ? local->health_state : YAI_DAEMON_HEALTH_STARTING,
               local->binding_count,
               local->spool_queued,
               local->spool_retry_due,
               local->spool_delivered,
               local->spool_failed,
               local->scan_discovered,
               local->emit_attempts,
               local->emit_success,
               local->emit_failures,
               local->owner_connected ? "true" : "false",
               local->owner_registered ? "true" : "false",
               (long long)local->last_owner_contact_epoch) >= (int)out_cap) return -1;
  return 0;
}

static int write_health_file(const yai_daemon_local_runtime_t *local)
{
  char payload[2048];
  if (!local || !g_paths) return -1;
  if (yai_daemon_local_runtime_health_json(local, payload, sizeof(payload)) != 0) return -1;
  return yai_daemon_write_file(g_paths->health_file, payload);
}

int yai_daemon_local_runtime_init(yai_daemon_local_runtime_t *local,
                                  const yai_daemon_config_t *cfg,
                                  const yai_daemon_paths_t *paths,
                                  const char *instance_id,
                                  const char *source_label)
{
  if (!local || !cfg || !paths || !instance_id || !source_label) return -1;
  memset(local, 0, sizeof(*local));
  g_cfg = cfg;
  g_paths = paths;
  snprintf(g_instance_id, sizeof(g_instance_id), "%s", instance_id);

  if (build_paths(local) != 0) return -1;
  if (parse_owner_socket(cfg->owner_ref, local->owner_socket, sizeof(local->owner_socket)) != 0) {
    local->owner_socket[0] = '\0';
  }
  if (yai_source_id_node(local->source_node_id, sizeof(local->source_node_id), source_label) != 0) return -1;
  if (yai_source_id_daemon_instance(local->daemon_instance_id, sizeof(local->daemon_instance_id), local->source_node_id) != 0) return -1;
  if (cfg->owner_ref[0]) (void)yai_source_id_owner_link(local->owner_link_id, sizeof(local->owner_link_id), local->source_node_id, cfg->owner_ref);
  snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_STARTING);
  (void)load_observed_index(local);
  return 0;
}

int yai_daemon_local_runtime_start(yai_daemon_local_runtime_t *local)
{
  size_t i = 0;
  if (!local) return -1;
  if (load_bindings_manifest(local) != 0) {
    snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DEGRADED);
    return -1;
  }
  for (i = 0; i < local->binding_count; ++i) {
    struct stat st;
    if (!local->bindings[i].enabled) {
      snprintf(local->bindings[i].status, sizeof(local->bindings[i].status), "%s", YAI_DAEMON_BINDING_STATUS_INVALID);
      continue;
    }
    if (stat(local->bindings[i].root_path, &st) != 0 || !S_ISDIR(st.st_mode)) {
      snprintf(local->bindings[i].status, sizeof(local->bindings[i].status), "%s", YAI_DAEMON_BINDING_STATUS_DEGRADED);
      continue;
    }
    snprintf(local->bindings[i].status, sizeof(local->bindings[i].status), "%s", YAI_DAEMON_BINDING_STATUS_ACTIVE);
  }
  (void)write_bindings_state(local);
  if (local->binding_count == 0) snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DEGRADED);
  else if (local->owner_socket[0] == '\0') snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DISCONNECTED);
  else snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_READY);
  (void)refresh_spool_counts(local);
  (void)write_health_file(local);
  return 0;
}

int yai_daemon_local_runtime_tick(yai_daemon_local_runtime_t *local, uint32_t tick_count)
{
  size_t i;
  (void)tick_count;
  if (!local) return -1;
  for (i = 0; i < local->binding_count; ++i) {
    if (local->bindings[i].enabled && strcmp(local->bindings[i].status, YAI_DAEMON_BINDING_STATUS_INVALID) != 0) {
      (void)scan_binding(local, &local->bindings[i]);
    }
  }
  (void)persist_observed_index(local);
  (void)process_queue(local);
  (void)refresh_spool_counts(local);
  local->spool_retry_due = local->spool_queued;
  if (local->owner_socket[0] == '\0') {
    snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DISCONNECTED);
  } else if (local->owner_connected) {
    snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_READY);
  } else if (local->spool_queued > 0) {
    snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DEGRADED);
  } else {
    snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_DISCONNECTED);
  }
  if ((tick_count % 5U) == 0U && local->binding_count > 0) {
    (void)send_status_update(local, local->bindings[0].workspace_id);
  }
  (void)write_bindings_state(local);
  (void)write_health_file(local);
  return 0;
}

int yai_daemon_local_runtime_stop(yai_daemon_local_runtime_t *local)
{
  if (!local) return -1;
  snprintf(local->health_state, sizeof(local->health_state), "%s", YAI_DAEMON_HEALTH_STOPPING);
  (void)refresh_spool_counts(local);
  (void)write_health_file(local);
  return 0;
}

/* SPDX-License-Identifier: Apache-2.0 */
// src/rpc/rpc_client.c

#define _POSIX_C_SOURCE 200809L

#include <yai/sdk/rpc.h>
#include <yai/sdk/paths.h>

#include <yai/protocol/rpc/contract.h>         /* yai_handshake_req_t / yai_handshake_ack_t */
#include <yai/protocol/transport/transport.h>        /* yai_rpc_envelope_t + frame constants */
#include <yai/protocol/control/ids.h> /* YAI_PROTOCOL_IDS_VERSION + command ids */
#include <yai/protocol/control/roles.h>            /* YAI_ROLE_* */

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include <stddef.h>
#include <stdint.h>
#include <ctype.h>

#include "../platform/log_internal.h"

/* ============================================================
   INTERNAL IO (STRICT)
   ============================================================ */

static int write_all(int fd, const void *buf, size_t n)
{
    const uint8_t *p = (const uint8_t *)buf;
    size_t off = 0;

    while (off < n)
    {
        ssize_t w = write(fd, p + off, n - off);
        if (w < 0)
        {
            if (errno == EINTR)
                continue;
            return -1;
        }
        if (w == 0)
            return -1;
        off += (size_t)w;
    }
    return 0;
}

static int read_all(int fd, void *buf, size_t n)
{
    uint8_t *p = (uint8_t *)buf;
    size_t off = 0;

    while (off < n)
    {
        ssize_t r = read(fd, p + off, n - off);
        if (r < 0)
        {
            if (errno == EINTR)
                continue;
            return -1;
        }
        if (r == 0)
            return -1; /* EOF */
        off += (size_t)r;
    }
    return 0;
}

/* ============================================================
   WS_ID VALIDATION (STRICT, PATH-SAFE)
   - must fit envelope scope_id field (typically 36 bytes incl NUL)
   - allow: [A-Za-z0-9_-]
   ============================================================ */

static int is_valid_scope_id(const char *scope_id)
{
    if (!scope_id || !scope_id[0])
        return 0;

    /* forbid traversal / separators / shortcuts */
    if (strchr(scope_id, '/'))
        return 0;
    if (scope_id[0] == '~')
        return 0;
    if (strstr(scope_id, ".."))
        return 0;

    /* allow only safe charset; enforce max length */
    size_t n = 0;
    for (const char *p = scope_id; *p; p++)
    {
        unsigned char c = (unsigned char)*p;
        if (!(isalnum(c) || c == '_' || c == '-'))
            return 0;
        n++;
        /* envelope scope_id is usually 36 bytes -> 35 chars max */
        if (n > 35)
            return 0;
    }

    return 1;
}

/* ============================================================
   TRACE ID (deterministic enough for local debug)
   NOTE: envelope trace_id is typically 36 bytes -> keep short
   ============================================================ */

static void set_trace_id(yai_rpc_client_t *c, yai_rpc_envelope_t *env)
{
    const char *prefix = "sdk";
    uint32_t ctr = 0;
    unsigned long pid = 0;

    if (!c || !env)
        return;

    if (c->correlation_id[0])
        prefix = c->correlation_id;

    c->trace_seq++;
    ctr = c->trace_seq;
    pid = (unsigned long)getpid();

    (void)snprintf(env->trace_id, sizeof(env->trace_id), "%.16s-%lu-%u",
                   prefix, pid, ctr);
}

/* ============================================================
   CONNECT / CLOSE
   ============================================================ */

int yai_rpc_connect_at(yai_rpc_client_t *c, const char *scope_id, const char *sock_path)
{
    if (!c)
        return -1;
    if (!is_valid_scope_id(scope_id))
        return -99;
    if (!sock_path || !sock_path[0])
        return -2;

    memset(c, 0, sizeof(*c));
    c->fd = -1;
    c->trace_seq = 0;

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0)
        return -3;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;

    if (strlen(sock_path) >= sizeof(addr.sun_path))
    {
        close(fd);
        return -4; /* path too long for AF_UNIX */
    }
    if (snprintf(addr.sun_path, sizeof(addr.sun_path), "%s", sock_path) < 0)
    {
        close(fd);
        return -4;
    }

    socklen_t len =
        (socklen_t)(offsetof(struct sockaddr_un, sun_path) + strlen(addr.sun_path));

    if (connect(fd, (struct sockaddr *)&addr, len) < 0)
    {
        yai_sdk_log_emit(YAI_SDK_LOG_ERROR, "rpc", "connect failed");
        close(fd);
        return -5;
    }

    c->fd = fd;

    if (snprintf(c->scope_id, sizeof(c->scope_id), "%s", scope_id) < 0)
    {
        close(fd);
        c->fd = -1;
        return -2;
    }

    /* default authority (explicitly NONE) */
    c->role = YAI_ROLE_NONE;
    c->arming = 0;
    c->correlation_id[0] = '\0';

    return 0;
}

int yai_rpc_connect(yai_rpc_client_t *c, const char *scope_id)
{
    char sock_path[512];
    if (yai_path_runtime_ingress_sock(sock_path, sizeof(sock_path)) != 0)
        return -2;
    return yai_rpc_connect_at(c, scope_id, sock_path);
}

void yai_rpc_close(yai_rpc_client_t *c)
{
    if (c && c->fd >= 0)
    {
        close(c->fd);
        c->fd = -1;
    }
}

/* ============================================================
   AUTHORITY
   ============================================================ */

void yai_rpc_set_authority(yai_rpc_client_t *c, int arming, const char *role_str)
{
    if (!c)
        return;

    c->arming = arming ? 1 : 0;

    if (!role_str || !role_str[0])
    {
        c->role = YAI_ROLE_NONE;
        return;
    }

    if (strcmp(role_str, "operator") == 0)
    {
        c->role = YAI_ROLE_OPERATOR;
    }
    else if (strcmp(role_str, "system") == 0)
    {
        c->role = YAI_ROLE_SYSTEM;
    }
    else if (strcmp(role_str, "user") == 0)
    {
        /* only if your roles.h defines it; otherwise drop this branch */
        c->role = YAI_ROLE_USER;
    }
    else
    {
        c->role = YAI_ROLE_NONE;
    }
}

/* ============================================================
   RAW CALL (envelope + payload, strict)
   ============================================================ */

int yai_rpc_call_raw(
    yai_rpc_client_t *c,
    uint32_t command_id,
    const void *payload,
    uint32_t payload_len,
    void *out_buf,
    size_t out_cap,
    uint32_t *out_len)
{
    if (!c || c->fd < 0)
        return -1;
    if (payload_len > 0 && !payload)
        return -2;

    yai_rpc_envelope_t env;
    memset(&env, 0, sizeof(env));

    env.magic = YAI_FRAME_MAGIC;
    env.version = YAI_PROTOCOL_IDS_VERSION;
    env.command_id = command_id;
    env.payload_len = payload_len;

    /* AUTHORITY (THIS IS THE FIX) */
    env.role = c->role;
    env.arming = c->arming;

    /* reserved; keep deterministic */
    env.checksum = 0;

    if (snprintf(env.ws_id, sizeof(env.ws_id), "%.*s", (int)sizeof(env.ws_id) - 1, c->scope_id) < 0)
        return -11;
    set_trace_id(c, &env);

    if (write_all(c->fd, &env, sizeof(env)) != 0)
        return -3;

    if (payload_len > 0)
    {
        if (write_all(c->fd, payload, payload_len) != 0)
            return -4;
    }

    yai_rpc_envelope_t resp;
    memset(&resp, 0, sizeof(resp));

    if (read_all(c->fd, &resp, sizeof(resp)) != 0)
        return -5;

    if (resp.magic != YAI_FRAME_MAGIC)
        return -6;

    if (resp.version != YAI_PROTOCOL_IDS_VERSION)
        return -7;

    if (resp.payload_len > (uint32_t)out_cap)
        return -8;

    if (resp.payload_len > 0)
    {
        if (!out_buf)
            return -9;
        if (read_all(c->fd, out_buf, resp.payload_len) != 0)
            return -10;
    }

    if (out_len)
        *out_len = resp.payload_len;
    return 0;
}

/* ============================================================
   HANDSHAKE (protocol.h structs)
   ============================================================ */

int yai_rpc_handshake(yai_rpc_client_t *c)
{
    if (!c || c->fd < 0)
        return -1;

    yai_handshake_req_t req;
    memset(&req, 0, sizeof(req));
    req.client_version = YAI_PROTOCOL_IDS_VERSION;
    req.capabilities_requested = 0;
    (void)snprintf(req.client_name, sizeof(req.client_name), "%s", "yai-shell");

    yai_handshake_ack_t ack;
    memset(&ack, 0, sizeof(ack));

    uint32_t out_len = 0;

    int rc = yai_rpc_call_raw(
        c,
        YAI_CMD_HANDSHAKE,
        &req,
        (uint32_t)sizeof(req),
        &ack,
        sizeof(ack),
        &out_len);

    if (rc != 0)
        return rc;
    if (out_len != (uint32_t)sizeof(ack))
        return -20;
    if (ack.server_version != YAI_PROTOCOL_IDS_VERSION)
        return -21;
    if (ack.status != YAI_PROTO_STATE_READY)
        return -22;

    return 0;
}
void yai_rpc_set_correlation_id(yai_rpc_client_t *c, const char *correlation_id)
{
    if (!c) return;
    if (!correlation_id || !correlation_id[0]) {
        c->correlation_id[0] = '\0';
        return;
    }
    (void)snprintf(c->correlation_id, sizeof(c->correlation_id), "%s", correlation_id);
}

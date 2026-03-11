#include <yai/orchestration/transport_client.h>
#include <yai/api/runtime.h>

#include "protocol.h"
#include "yai_protocol_ids.h"

#include <errno.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

static int write_all(int fd, const void *buf, size_t len)
{
    const char *p = (const char *)buf;
    size_t off = 0;

    while (off < len)
    {
        ssize_t written = write(fd, p + off, len - off);
        if (written < 0)
        {
            if (errno == EINTR)
            {
                continue;
            }
            return -1;
        }
        off += (size_t)written;
    }

    return 0;
}

static int read_all(int fd, void *buf, size_t len)
{
    char *p = (char *)buf;
    size_t off = 0;

    while (off < len)
    {
        ssize_t read_count = read(fd, p + off, len - off);
        if (read_count <= 0)
        {
            if (read_count < 0 && errno == EINTR)
            {
                continue;
            }
            return -1;
        }
        off += (size_t)read_count;
    }

    return 0;
}

void yai_make_trace_id(char out[36])
{
    static uint32_t counter = 0;

    if (!out)
    {
        return;
    }

    snprintf(out, 36, "tr-%lx-%u", (unsigned long)time(NULL), counter++);
}

int yai_runtime_ingress_path(char *out, uint32_t out_cap)
{
    size_t cap = (size_t)out_cap;
    const char *home = getenv("HOME");
    const char *override = getenv(YAI_RUNTIME_INGRESS_ENV);

    if (!out || cap == 0)
    {
        return -1;
    }

    if (override && override[0])
    {
        if (snprintf(out, cap, "%s", override) >= (int)cap)
        {
            return -2;
        }
        return 0;
    }

    if (!home || !home[0])
    {
        return -3;
    }

    if (snprintf(out, cap, "%s/%s", home, YAI_RUNTIME_INGRESS_SOCKET_REL) >= (int)cap)
    {
        return -4;
    }

    return 0;
}

int yai_runtime_peer_ingress_path(char *out, uint32_t out_cap)
{
    size_t cap = (size_t)out_cap;
    const char *home = getenv("HOME");
    const char *override = getenv(YAI_RUNTIME_PEER_INGRESS_ENV);

    if (!out || cap == 0)
    {
        return -1;
    }

    if (override && override[0])
    {
        if (snprintf(out, cap, "%s", override) >= (int)cap)
        {
            return -2;
        }
        return 0;
    }

    if (!home || !home[0])
    {
        return -3;
    }

    if (snprintf(out, cap, "%s/%s", home, YAI_RUNTIME_PEER_INGRESS_SOCKET_REL) >= (int)cap)
    {
        return -4;
    }

    return 0;
}

int yai_rpc_connect_at_ingress(yai_rpc_client_t *client, const char *ws_id, const char *socket_path)
{
    struct sockaddr_un addr;
    socklen_t addr_len = 0;
    int fd = -1;

    if (!client || !socket_path || !socket_path[0])
    {
        return -1;
    }

    memset(client, 0, sizeof(*client));

    fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0)
    {
        return -2;
    }

    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;

    strncpy(addr.sun_path, socket_path, sizeof(addr.sun_path) - 1);
    addr.sun_path[sizeof(addr.sun_path) - 1] = '\0';

    addr_len = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + strlen(addr.sun_path));

    if (connect(fd, (struct sockaddr *)&addr, addr_len) < 0)
    {
        close(fd);
        return -3;
    }

    client->fd = fd;

    if (!ws_id || !ws_id[0] || strchr(ws_id, '/'))
    {
        ws_id = "system";
    }

    strncpy(client->ws_id, ws_id, sizeof(client->ws_id) - 1);
    client->ws_id[sizeof(client->ws_id) - 1] = '\0';
    client->connected = true;

    return 0;
}

int yai_rpc_connect(yai_rpc_client_t *client, const char *ws_id)
{
    char socket_path[256];

    if (yai_runtime_ingress_path(socket_path, (uint32_t)sizeof(socket_path)) != 0)
    {
        return -1;
    }
    return yai_rpc_connect_at_ingress(client, ws_id, socket_path);
}

int yai_rpc_call(yai_rpc_client_t *client,
                 uint32_t command_id,
                 const void *payload,
                 uint32_t payload_len,
                 void *out_buf,
                 uint32_t out_cap,
                 uint32_t *out_len)
{
    yai_rpc_envelope_t request;
    yai_rpc_envelope_t response;

    if (!client || !client->connected)
    {
        return -1;
    }

    if (payload_len > YAI_MAX_PAYLOAD)
    {
        return -2;
    }

    memset(&request, 0, sizeof(request));
    request.magic = YAI_FRAME_MAGIC;
    request.version = YAI_PROTOCOL_IDS_VERSION;
    request.command_id = command_id;
    request.payload_len = payload_len;

    strncpy(request.ws_id, client->ws_id, sizeof(request.ws_id) - 1);
    request.ws_id[sizeof(request.ws_id) - 1] = '\0';
    yai_make_trace_id(request.trace_id);

    if (write_all(client->fd, &request, sizeof(request)) != 0)
    {
        return -3;
    }

    if (payload_len > 0 && payload)
    {
        if (write_all(client->fd, payload, payload_len) != 0)
        {
            return -4;
        }
    }

    if (read_all(client->fd, &response, sizeof(response)) != 0)
    {
        return -5;
    }

    if (response.magic != YAI_FRAME_MAGIC)
    {
        return -6;
    }

    if (response.version != YAI_PROTOCOL_IDS_VERSION)
    {
        return -7;
    }

    if (response.payload_len > out_cap)
    {
        return -8;
    }

    if (response.payload_len > 0 && out_buf)
    {
        if (read_all(client->fd, out_buf, response.payload_len) != 0)
        {
            return -9;
        }
    }

    if (out_len)
    {
        *out_len = response.payload_len;
    }

    return 0;
}

int yai_rpc_handshake(yai_rpc_client_t *client, uint32_t capabilities)
{
    yai_handshake_req_t request;
    uint8_t response_buf[64];
    uint32_t response_len = 0;
    int rc = 0;

    if (!client || !client->connected)
    {
        return -1;
    }

    memset(&request, 0, sizeof(request));
    request.client_version = YAI_PROTOCOL_IDS_VERSION;
    request.capabilities_requested = capabilities;
    strncpy(request.client_name, "yai", sizeof(request.client_name) - 1);
    request.client_name[sizeof(request.client_name) - 1] = '\0';

    rc = yai_rpc_call(client,
                      YAI_CMD_HANDSHAKE,
                      &request,
                      sizeof(request),
                      response_buf,
                      sizeof(response_buf),
                      &response_len);
    if (rc != 0)
    {
        return rc;
    }

    if (response_len != sizeof(yai_handshake_ack_t))
    {
        return -2;
    }

    yai_handshake_ack_t *ack = (yai_handshake_ack_t *)response_buf;
    if (ack->status != YAI_PROTO_STATE_READY)
    {
        return -3;
    }

    return 0;
}

void yai_rpc_close(yai_rpc_client_t *client)
{
    if (!client || !client->connected)
    {
        return;
    }

    close(client->fd);
    client->connected = false;
}

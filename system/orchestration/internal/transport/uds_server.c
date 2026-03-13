/* SPDX-License-Identifier: Apache-2.0 */

#include <yai/orchestration/transport.h>

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

static const char *g_default_socket = "/tmp/yai-runtime.sock";

static int write_all(int fd, const char *buf, size_t len)
{
  size_t written = 0;
  while (written < len) {
    ssize_t n = write(fd, buf + written, len - written);
    if (n <= 0) return YAI_MIND_ERR_TRANSPORT;
    written += (size_t)n;
  }
  return YAI_MIND_OK;
}

const char *yai_uds_default_path(void)
{
  return g_default_socket;
}

int yai_transport_handle_raw(const char *raw,
                                  char *response_out,
                                  size_t response_cap)
{
  yai_protocol_request_t request = {0};
  yai_protocol_response_t response = {0};
  int rc;

  if (!raw || !response_out || response_cap == 0) return YAI_MIND_ERR_INVALID_ARG;
  if (!yai_transport_is_initialized()) return YAI_MIND_ERR_STATE;

  rc = yai_protocol_parse(raw, &request);
  if (rc != YAI_MIND_OK) {
    response.status = 400;
    response.code = rc;
    snprintf(response.payload, sizeof(response.payload), "protocol parse failed");
    (void)yai_protocol_format_response(&response, response_out, response_cap);
    return rc;
  }

  rc = yai_protocol_dispatch(&request, &response);
  (void)yai_protocol_format_response(&response, response_out, response_cap);
  return rc;
}

int yai_uds_server_run_once(const char *socket_path)
{
  int server_fd = -1;
  int client_fd = -1;
  struct sockaddr_un addr;
  char req_buf[1024];
  char resp_buf[1024];
  ssize_t nread;
  const char *path = socket_path && socket_path[0] ? socket_path : yai_uds_default_path();
  int wrc;

  if (!yai_transport_is_initialized()) return YAI_MIND_ERR_STATE;

  server_fd = socket(AF_UNIX, SOCK_STREAM, 0);
  if (server_fd < 0) return YAI_MIND_ERR_TRANSPORT;

  memset(&addr, 0, sizeof(addr));
  addr.sun_family = AF_UNIX;
  if (snprintf(addr.sun_path, sizeof(addr.sun_path), "%s", path) >= (int)sizeof(addr.sun_path)) {
    close(server_fd);
    return YAI_MIND_ERR_INVALID_ARG;
  }
  unlink(path);

  if (bind(server_fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
    close(server_fd);
    return YAI_MIND_ERR_TRANSPORT;
  }

  if (listen(server_fd, 1) != 0) {
    close(server_fd);
    unlink(path);
    return YAI_MIND_ERR_TRANSPORT;
  }

  client_fd = accept(server_fd, NULL, NULL);
  if (client_fd < 0) {
    close(server_fd);
    unlink(path);
    return YAI_MIND_ERR_TRANSPORT;
  }

  nread = read(client_fd, req_buf, sizeof(req_buf) - 1);
  if (nread <= 0) {
    close(client_fd);
    close(server_fd);
    unlink(path);
    return YAI_MIND_ERR_TRANSPORT;
  }
  req_buf[nread] = '\0';

  (void)yai_transport_handle_raw(req_buf, resp_buf, sizeof(resp_buf));
  wrc = write_all(client_fd, resp_buf, strlen(resp_buf));

  close(client_fd);
  close(server_fd);
  unlink(path);
  return wrc;
}

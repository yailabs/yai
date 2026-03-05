/* SPDX-License-Identifier: Apache-2.0 */
#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <limits.h>
#include <time.h>
#include <errno.h>
#include <sys/socket.h>

#include <transport.h>
#include <yai_protocol_ids.h>

#include "control_transport.h"
#include "root_command_dispatch.h"

static FILE *root_log = NULL;

/* ============================================================
   LOGGING
   ============================================================ */

#define LOG(fmt, ...)                                     \
    do {                                                  \
        fprintf(stdout, fmt "\n", ##__VA_ARGS__);         \
        if (root_log && root_log != stdout)               \
            fprintf(root_log, fmt "\n", ##__VA_ARGS__);   \
        fflush(stdout);                                   \
        if (root_log && root_log != stdout)               \
            fflush(root_log);                             \
    } while (0)

static void log_init(const char *home)
{
    char path[PATH_MAX];

    snprintf(path, sizeof(path),
             "%s/.yai/run/root/root.log", home);

    root_log = fopen(path, "a");

    if (!root_log) {
        fprintf(stderr,
                "[ROOT] Failed to open log file: %s (%s)\n",
                path, strerror(errno));
        root_log = stdout;
    } else {
        setvbuf(root_log, NULL, _IOLBF, 0);
    }

    time_t now = time(NULL);
    LOG("\n=== ROOT START %ld ===", now);
}

/* ============================================================
   HANDLE CLIENT
   ============================================================ */

static void handle_client(int cfd)
{
    LOG("[ROOT] Client connected");

    int handshake_done = 0;

    for (;;) {

        yai_rpc_envelope_t env;
        char payload[YAI_MAX_PAYLOAD];

        /* IMPORTANT:
           yai_control_read_frame() returns payload_len.
           payload_len can be 0 and that is VALID.
        */
        ssize_t plen = yai_control_read_frame(cfd, &env, payload, sizeof(payload));

        if (plen < 0) {
            if (plen == YAI_CTL_ERR_OVERFLOW) {
                LOG("[ROOT] Reject overflow payload");
            }
            break;
        }


        int rc = root_dispatch_frame(cfd, &env, payload, plen, &handshake_done);
        if (rc != 0) {
            LOG("[ROOT] Command rejected/failed cmd=%u rc=%d", env.command_id, rc);
            break;
        }
    }

    close(cfd);
    LOG("[ROOT] Client disconnected");
}

/* ============================================================
   MAIN
   ============================================================ */

int main(void)
{
    const char *home = getenv("HOME");
    if (!home)
        home = "/tmp";

    log_init(home);

    char sock_path[PATH_MAX];

    snprintf(sock_path, sizeof(sock_path),
             "%s/.yai/run/root/root.sock", home);

    unlink(sock_path);

    int sfd = yai_control_listen_at(sock_path);
    if (sfd < 0) {
        LOG("[ROOT] Failed to bind socket: %s (%s)",
            sock_path, strerror(errno));
        return 1;
    }

    LOG("[ROOT] Listening on %s", sock_path);

    for (;;) {
        int cfd = accept(sfd, NULL, NULL);
        if (cfd >= 0)
            handle_client(cfd);
    }

    return 0;
}

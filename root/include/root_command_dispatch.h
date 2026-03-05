#ifndef ROOT_COMMAND_DISPATCH_H
#define ROOT_COMMAND_DISPATCH_H

#include <sys/types.h>
#include <protocol.h>

int root_dispatch_frame(
    int client_fd,
    const yai_rpc_envelope_t *env,
    const char *payload,
    ssize_t payload_len,
    int *handshake_done);

#endif

#pragma once

#ifndef INCLUDE_IPC_RECORD_H
#define INCLUDE_IPC_RECORD_H

#include <ipc/types.h>
#include <ipc/session_state.h>

struct yai_ipc_record {
    yai_ipc_session_id_t session_id;
    enum yai_ipc_session_state state;
    const char *role;
};

#endif

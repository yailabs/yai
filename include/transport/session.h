#pragma once

#ifndef INCLUDE_TRANSPORT_SESSION_H
#define INCLUDE_TRANSPORT_SESSION_H

#include <stdint.h>

typedef uint64_t yai_transport_session_id_t;

struct yai_transport_session {
    yai_transport_session_id_t session_id;
    unsigned int flags;
};

#endif

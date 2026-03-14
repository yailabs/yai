#pragma once

#ifndef INCLUDE_TRANSPORT_RECORD_H
#define INCLUDE_TRANSPORT_RECORD_H

#include <transport/ids.h>
#include <transport/state.h>

struct yai_transport_record {
    yai_transport_id_t transport_id;
    enum yai_transport_state state;
    const char *name;
};

#endif

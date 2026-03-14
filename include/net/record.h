#pragma once

#ifndef INCLUDE_NET_RECORD_H
#define INCLUDE_NET_RECORD_H

#include <net/types.h>

struct yai_net_record {
    yai_net_peer_id_t peer_id;
    enum yai_net_plane_kind plane;
    const char *label;
};

#endif

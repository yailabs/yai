#pragma once

#ifndef INCLUDE_NS_RECORD_H
#define INCLUDE_NS_RECORD_H

#include <yai/container/ns/types.h>
#include <yai/container/ns/state.h>

struct yai_ns_record {
    yai_ns_id_t ns_id;
    enum yai_ns_kind kind;
    enum yai_ns_state state;
    const char *name;
};

#endif

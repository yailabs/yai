#pragma once

#ifndef INCLUDE_CONTAINER_CONTAINER_RECORD_H
#define INCLUDE_CONTAINER_CONTAINER_RECORD_H

struct yai_container_record {
    const char *name;
    enum yai_container_state state;
    enum yai_container_mode mode;
};

#endif

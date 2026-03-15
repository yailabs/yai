#pragma once

#ifndef INCLUDE_MOUNT_NAMESPACE_BINDING_H
#define INCLUDE_MOUNT_NAMESPACE_BINDING_H

#include <mount/types.h>
#include <yai/container/ns/types.h>

struct yai_mount_namespace_binding {
    yai_mount_graph_id_t graph_id;
    yai_ns_id_t ns_id;
};

#endif

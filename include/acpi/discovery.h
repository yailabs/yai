#pragma once

#ifndef INCLUDE_ACPI_DISCOVERY_H
#define INCLUDE_ACPI_DISCOVERY_H

#include <acpi/types.h>

struct yai_acpi_discovery_record {
    yai_acpi_node_id_t node_id;
    enum yai_acpi_object_kind kind;
    const char *hid;
};

#endif

#pragma once

#ifndef INCLUDE_ACPI_TYPES_H
#define INCLUDE_ACPI_TYPES_H

#include <stdint.h>

typedef uint64_t yai_acpi_table_id_t;
typedef uint64_t yai_acpi_node_id_t;

enum yai_acpi_object_kind {
    YAI_ACPI_OBJECT_UNKNOWN = 0,
    YAI_ACPI_OBJECT_DEVICE,
    YAI_ACPI_OBJECT_PROCESSOR,
    YAI_ACPI_OBJECT_THERMAL,
    YAI_ACPI_OBJECT_POWER
};

#endif

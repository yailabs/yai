#pragma once

#ifndef INCLUDE_ACPI_TABLES_HEADER_H
#define INCLUDE_ACPI_TABLES_HEADER_H

#include <stdint.h>

struct yai_acpi_table_header {
    char signature[4];
    uint32_t length;
    uint8_t revision;
};

#endif

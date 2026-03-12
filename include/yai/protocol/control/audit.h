/**
 * @file audit.h
 * @brief Audit record schema for protocol events.
 */
#ifndef YAI_PROTOCOL_AUDIT_H
#define YAI_PROTOCOL_AUDIT_H

#include <stdint.h>
#include <stddef.h>

#pragma pack(push, 1)
/** Packed audit record. */
typedef struct yai_audit_record {
    uint32_t event_id;
    uint32_t state_from;
    uint32_t state_to;
    uint32_t command_id;
    uint64_t timestamp_ns;
    uint64_t logical_clock;
    uint32_t seq;
    int32_t  result_code;
    uint32_t reserved;
} yai_audit_record_t;
#pragma pack(pop)

_Static_assert(sizeof(yai_audit_record_t) == 44, "yai_audit_record_t size mismatch");

#endif /* YAI_PROTOCOL_AUDIT_H */

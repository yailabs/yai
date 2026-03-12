/* SPDX-License-Identifier: Apache-2.0 */
/**
 * @file transport.h
 * @brief RPC envelope and transport framing constants.
 *
 * Protocol-layer transport contract shared by core/exec/brain runtime planes.
 */
#ifndef YAI_PROTOCOL_TRANSPORT_H
#define YAI_PROTOCOL_TRANSPORT_H

#include <stdint.h>

#define YAI_FRAME_MAGIC 0x59414950u
#define YAI_MAX_PAYLOAD 65536u

#pragma pack(push, 1)

/** RPC transport envelope. */
typedef struct yai_rpc_envelope {
    uint32_t magic;
    uint32_t version;

    char     ws_id[36];
    char     trace_id[36];

    uint32_t command_id;
    uint16_t role;
    uint8_t  arming;
    uint8_t  _pad;

    uint32_t payload_len;
    uint32_t checksum;

} yai_rpc_envelope_t;

#pragma pack(pop)

_Static_assert(sizeof(yai_rpc_envelope_t) == 96,
               "yai_rpc_envelope_t must be 96 bytes");

#endif

/* SPDX-License-Identifier: Apache-2.0 */
/**
 * @file ids.h
 * @brief Canonical protocol command classes and IDs.
 */
#ifndef YAI_PROTOCOL_IDS_H
#define YAI_PROTOCOL_IDS_H

#define YAI_PROTOCOL_IDS_VERSION 1

/* ============================================================
   COMMAND CLASSES
   ============================================================ */

#define YAI_CMD_CLASS_CONTROL     0x0100u
#define YAI_CMD_CLASS_STORAGE     0x0200u
#define YAI_CMD_CLASS_PROVIDER    0x0300u
#define YAI_CMD_CLASS_PRIVILEGED  0xF000u

/* ============================================================
   COMMAND IDS
   ============================================================ */

/** Command IDs (never reuse). */
typedef enum {

    /* Control Plane (0x01xx) */
    YAI_CMD_PING        = 0x0101u,
    YAI_CMD_HANDSHAKE   = 0x0102u,
    YAI_CMD_CONTROL     = 0x0104u,
    YAI_CMD_CONTROL_CALL = 0x0105u,

    /* Privileged */
    YAI_CMD_RECONFIGURE = (0x0103u | YAI_CMD_CLASS_PRIVILEGED),

    /* Storage */
    YAI_CMD_STORAGE_RPC = 0x0201u,

    /* Providers */
    YAI_CMD_PROVIDER_RPC  = 0x0301u,
    YAI_CMD_EMBEDDING_RPC = 0x0302u

} yai_command_id_t;

#endif /* YAI_PROTOCOL_IDS_H */

/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stdint.h>

#include <yai/ipc/message_types.h>
#include <yai/ipc/transport.h>

/* Initial binary framing contract placeholder; to be consolidated in protocol refoundation wave. */
typedef struct yai_rpc_binary_frame {
  uint32_t length;
  uint16_t version;
  uint16_t flags;
} yai_rpc_binary_frame_t;

int yai_rpc_binary_validate_envelope(const yai_rpc_binary_frame_t *frame);

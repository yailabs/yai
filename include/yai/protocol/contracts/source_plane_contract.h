/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <yai/edge/source_plane_model.h>

#define YAI_SOURCE_CONTRACT_TYPE_ENROLL_CALL "yai.source.enroll.call.v1"
#define YAI_SOURCE_CONTRACT_TYPE_ENROLL_REPLY "yai.source.enroll.reply.v1"
#define YAI_SOURCE_CONTRACT_TYPE_ATTACH_CALL "yai.source.attach.call.v1"
#define YAI_SOURCE_CONTRACT_TYPE_ATTACH_REPLY "yai.source.attach.reply.v1"
#define YAI_SOURCE_CONTRACT_TYPE_EMIT_CALL "yai.source.emit.call.v1"
#define YAI_SOURCE_CONTRACT_TYPE_EMIT_REPLY "yai.source.emit.reply.v1"
#define YAI_SOURCE_CONTRACT_TYPE_STATUS_CALL "yai.source.status.call.v1"
#define YAI_SOURCE_CONTRACT_TYPE_STATUS_REPLY "yai.source.status.reply.v1"

typedef struct yai_source_contract_shape {
  yai_source_contract_operation_t op;
  const char *call_type;
  const char *reply_type;
} yai_source_contract_shape_t;

const yai_source_contract_shape_t *yai_source_contract_shape(yai_source_contract_operation_t op);

/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

/* Canonical command-id surface exported from protocol spine. */
#include <yai/protocol/contracts/ids.h>
#include <yai/protocol/contracts/source_plane.h>

/* Source-plane transport intents (YD-4 baseline). */
#define YAI_MSG_SOURCE_ENROLL "SOURCE_ENROLL"
#define YAI_MSG_SOURCE_ATTACH "SOURCE_ATTACH"
#define YAI_MSG_SOURCE_EMIT "SOURCE_EMIT"
#define YAI_MSG_SOURCE_STATUS "SOURCE_STATUS"

unsigned int yai_protocol_ids_version(void);

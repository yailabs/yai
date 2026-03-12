/* SPDX-License-Identifier: Apache-2.0 */
/**
 * @file auth.h
 * @brief Authentication token schema and claims.
 */
#ifndef YAI_PROTOCOL_AUTH_H
#define YAI_PROTOCOL_AUTH_H

#include <stdint.h>
#include <stddef.h>

#define YAI_TOKEN_SUBJECT_MAX 32u
#define YAI_TOKEN_ISSUER_MAX  32u
#define YAI_TOKEN_NONCE_MAX   16u

/** Claim identifiers for auth tokens. */
typedef enum yai_claim_id {
    YAI_CLAIM_NONE = 0,
    YAI_CLAIM_SYSTEM = 1,
    YAI_CLAIM_OPERATOR = 2,
    YAI_CLAIM_AGENT = 3
} yai_claim_id_t;

#pragma pack(push, 1)
/** Packed authentication token. */
typedef struct yai_token {
    uint32_t version;
    uint32_t ttl_secs;
    uint64_t issued_at;
    uint32_t scope_bits;
    uint8_t  subject_id[YAI_TOKEN_SUBJECT_MAX];
    uint8_t  issuer_id[YAI_TOKEN_ISSUER_MAX];
    uint8_t  nonce[YAI_TOKEN_NONCE_MAX];
} yai_token_t;
#pragma pack(pop)

_Static_assert(sizeof(yai_token_t) == 100, "yai_token_t size mismatch");

#endif /* YAI_PROTOCOL_AUTH_H */

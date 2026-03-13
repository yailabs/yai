/**
 * @file roles.h
 * @brief Role identifiers for protocol envelopes.
 */
#ifndef YAI_PROTOCOL_ROLES_H
#define YAI_PROTOCOL_ROLES_H

/** Role constants for RPC envelopes. */
#define YAI_ROLE_NONE      0
#define YAI_ROLE_USER      1
#define YAI_ROLE_OPERATOR  2
#define YAI_ROLE_SYSTEM    3

/*
 * Roles are governance roles.
 * They are orthogonal to runtime plane naming (core/exec/brain) and
 * must not be interpreted as legacy package identities.
 */

#endif

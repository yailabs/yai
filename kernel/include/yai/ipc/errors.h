/* SPDX-License-Identifier: Apache-2.0 */
/**
 * @file errors.h
 * @brief Protocol error codes for transport and session handling.
 */
#ifndef YAI_PROTOCOL_ERRORS_H
#define YAI_PROTOCOL_ERRORS_H

/** Error codes returned by protocol handlers. */
typedef enum {
    YAI_OK = 0,
    YAI_E_BAD_MAGIC        = 100, // Non è un pacchetto YAI
    YAI_E_BAD_VERSION      = 101, // Protocollo non compatibile
    YAI_E_BAD_WS_ID        = 102, // Workspace ID illegale o pericoloso
    YAI_E_PAYLOAD_TOO_BIG  = 103, // Tentativo di buffer overflow o eccesso
    YAI_E_BAD_CHECKSUM     = 104, // Dati corrotti
    
    YAI_E_NEED_HANDSHAKE   = 200, // Sessione non inizializzata
    YAI_E_ARMING_REQUIRED  = 201, // Comando critico senza flag --arming
    YAI_E_ROLE_REQUIRED    = 202, // Privilegi insufficienti
    
    YAI_E_GOVERNANCE_VIOLATION    = 300, // IL CUORE DI YAI: Violazione della governance canonica
    
    YAI_E_INTERNAL_ERROR   = 500  // Crash o errore logica interna
} yai_error_t;

#endif

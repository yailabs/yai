/* NOTE: internal compatibility/tooling surface; not public-stable SDK API. */
// include/yai/sdk/registry/registry_registry_validate.h
#pragma once

#include "yai/sdk/registry/registry_types.h"

#ifdef __cplusplus
extern "C" {
#endif

// Full registry validation (cache + index semantics).
// Returns 0 OK; nonzero = violation.
int yai_law_registry_validate_all(const yai_law_registry_t* r);

// Validates a single command record (structure + artifact role refs).
int yai_law_registry_validate_command(const yai_law_registry_t* r, const yai_law_command_t* c);

#ifdef __cplusplus
}
#endif
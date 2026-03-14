#pragma once

#include <stddef.h>

int yai_exec_source_ingest_operation_known(const char *command_id);

/*
 * Owner-side source-plane ingest bridge.
 * Returns:
 *   0  success
 *  -2  bad args / structural payload issue
 *  -3  invalid state for operation
 *  -4  internal failure
 */
int yai_exec_source_ingest_handle(const char *workspace_id,
                                  const char *command_id,
                                  const char *payload_json,
                                  char *out_json,
                                  size_t out_cap,
                                  char *out_reason,
                                  size_t reason_cap);

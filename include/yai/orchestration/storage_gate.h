#pragma once

void yai_storage_init(void);
void yai_storage_shutdown(void);
int yai_exec_storage_gate_ready(void);
char *yai_storage_handle_rpc(const char *ws_id, const char *method, const char *params_json);

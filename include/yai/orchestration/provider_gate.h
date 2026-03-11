#pragma once

#include <yai/protocol/contracts/transport_contract.h>

typedef struct yai_provider_config {
  char id[32];
  char host[128];
  int port;
  char endpoint[128];
  char api_key[128];
} yai_provider_config_t;

void yai_provider_gate_init(const yai_provider_config_t *config);
char *yai_provider_gate_dispatch(const yai_rpc_envelope_t *env, const char *json_payload);

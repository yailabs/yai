#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef enum {
  CAP_FS_READ = 1 << 0,
  CAP_FS_WRITE = 1 << 1,
  CAP_NET_OUT = 1 << 2,
  CAP_PROCESS_SPAWN = 1 << 3,
  CAP_LLM_DIRECT = 1 << 4
} yai_agent_capability_t;

typedef struct {
  char agent_id[64];
  uint32_t capabilities;
  uint64_t memory_limit;
} yai_agent_contract_t;

bool yai_contract_check(const char *agent_id, yai_agent_capability_t cap);

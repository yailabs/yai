#pragma once

#include <stddef.h>

#include <yai/kernel/vault.h>
#include <yai/ipc/message_types.h>

int yai_generate_runtime_id(const yai_vault_t *vault, char *buffer, size_t cap);

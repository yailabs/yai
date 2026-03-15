#pragma once

#include <stddef.h>

#include <yai/con/runtime.h>

int yai_container_model_upsert(const yai_container_record_t *record);
int yai_container_model_get(const char *container_id, yai_container_record_t *out_record);
int yai_container_model_exists(const char *container_id);
int yai_container_model_remove(const char *container_id);

int yai_container_model_base_dir(char *out, size_t out_cap);
int yai_container_model_record_path(const char *container_id, char *out, size_t out_cap);

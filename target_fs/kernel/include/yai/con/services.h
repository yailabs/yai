#pragma once

#include <stddef.h>
#include <stdint.h>

#include "state.h"

#define YAI_CONTAINER_SERVICE_NAME_MAX 63u
#define YAI_CONTAINER_SERVICE_SURFACE_MAX 64u

typedef struct {
  char name[YAI_CONTAINER_SERVICE_NAME_MAX + 1u];
  uint64_t service_handle;
  yai_container_service_status_t status;
  int64_t updated_at;
} yai_container_service_entry_t;

typedef struct {
  uint64_t service_surface_handle;
  size_t total_services;
  size_t ready_services;
  size_t degraded_services;
  yai_container_service_entry_t entries[YAI_CONTAINER_SERVICE_SURFACE_MAX];
} yai_container_service_surface_t;

int yai_container_service_register(const char *container_id,
                                   const char *name,
                                   uint64_t service_handle);
int yai_container_service_mark_ready(const char *container_id, const char *name);
int yai_container_service_mark_degraded(const char *container_id, const char *name);
int yai_container_service_lookup(const char *container_id,
                                 const char *name,
                                 yai_container_service_entry_t *out_entry);
int yai_container_service_list(const char *container_id,
                               yai_container_service_entry_t *entries,
                               size_t cap,
                               size_t *out_len);
int yai_container_service_surface_get(const char *container_id,
                                      yai_container_service_surface_t *out_surface);

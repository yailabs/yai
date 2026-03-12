/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned long yai_desktop_surface_id_t;

typedef enum yai_desktop_surface_kind {
  YAI_DESKTOP_SURFACE_PANEL = 1,
  YAI_DESKTOP_SURFACE_OVERLAY = 2,
  YAI_DESKTOP_SURFACE_DIALOG = 3,
} yai_desktop_surface_kind_t;

typedef struct yai_desktop_surface_desc {
  yai_desktop_surface_id_t id;
  yai_desktop_surface_kind_t kind;
  const char *name;
  int x;
  int y;
  int w;
  int h;
  int visible;
} yai_desktop_surface_desc_t;

#ifdef __cplusplus
}
#endif

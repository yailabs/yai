/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include "surface.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned long yai_desktop_widget_id_t;

typedef struct yai_desktop_widget_ref {
  yai_desktop_widget_id_t id;
  yai_desktop_surface_id_t surface_id;
} yai_desktop_widget_ref_t;

#ifdef __cplusplus
}
#endif

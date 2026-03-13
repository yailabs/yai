/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct yai_desktop_theme {
  const char *name;
  unsigned int fg;
  unsigned int bg;
  unsigned int accent;
} yai_desktop_theme_t;

#ifdef __cplusplus
}
#endif

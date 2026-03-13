/* SPDX-License-Identifier: Apache-2.0 */
#pragma once

#include "backend.h"
#include "compositor.h"
#include "renderer.h"
#include "session.h"
#include "state.h"

#ifdef __cplusplus
extern "C" {
#endif

int yai_desktop_run(yai_desktop_session_t *session);

#ifdef __cplusplus
}
#endif

#ifndef YAI_KERNEL_SESSION_INTERNAL_H
#define YAI_KERNEL_SESSION_INTERNAL_H

#include "yai/kernel/session.h"

int yai_kernel_session_store(const struct yai_kernel_session* session);
int yai_kernel_session_load(yai_object_id_t session_id, struct yai_kernel_session* out_session);

#endif /* YAI_KERNEL_SESSION_INTERNAL_H */

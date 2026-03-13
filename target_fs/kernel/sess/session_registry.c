#include "yai/kernel/registry.h"
#include "internal.h"

int yai_kernel_session_store(const struct yai_kernel_session* session) {
    return yai_session_registry_upsert(session);
}

int yai_kernel_session_load(yai_object_id_t session_id, struct yai_kernel_session* out_session) {
    return yai_session_registry_get(session_id, out_session);
}

int yai_kernel_session_get(yai_object_id_t session_id, struct yai_kernel_session* out_session) {
    return yai_kernel_session_load(session_id, out_session);
}

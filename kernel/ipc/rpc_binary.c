#include <yai/ipc/binary.h>
#include <yai/ipc/message_types.h>

int yai_rpc_binary_validate_envelope(const yai_rpc_binary_frame_t *frame) {
  if (!frame) {
    return -1;
  }
  if (frame->version != YAI_PROTOCOL_IDS_VERSION) {
    return -1;
  }
  if (frame->length > YAI_MAX_PAYLOAD) {
    return -1;
  }
  return 0;
}

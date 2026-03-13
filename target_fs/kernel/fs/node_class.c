#include <yai/fs/node_class.h>

const char *yai_fs_node_class_name(yai_fs_node_class_t value) {
  switch (value) {
    case YAI_FS_NODE_FILE: return "file";
    case YAI_FS_NODE_DIRECTORY: return "directory";
    case YAI_FS_NODE_MOUNT: return "mount";
    case YAI_FS_NODE_UNKNOWN:
    default: return "unknown";
  }
}

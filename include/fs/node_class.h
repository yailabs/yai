#pragma once

typedef enum {
  YAI_FS_NODE_UNKNOWN = 0,
  YAI_FS_NODE_FILE,
  YAI_FS_NODE_DIRECTORY,
  YAI_FS_NODE_MOUNT,
} yai_fs_node_class_t;

const char *yai_fs_node_class_name(yai_fs_node_class_t value);

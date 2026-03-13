#include <errno.h>
#include <sys/stat.h>

#include <yai/hal/fs.h>

int yai_fs_path_exists(const char *path) {
  struct stat st;
  if (!path) {
    return 0;
  }
  return stat(path, &st) == 0;
}

int yai_fs_ensure_dir(const char *path) {
  if (!path) {
    return -1;
  }
  if (mkdir(path, 0755) == 0 || errno == EEXIST) {
    return 0;
  }
  return -1;
}

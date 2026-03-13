#include <unistd.h>

#include <yai/hal/os.h>

pid_t yai_os_getpid(void) {
  return getpid();
}

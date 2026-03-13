#include <yai/services/dispatch.h>

#include <sys/socket.h>
#include <sys/un.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include <stddef.h>

/* ---------------------------
   Internal helpers
---------------------------- */
static int read_all(int fd, void *buf, size_t n) {
    char *p = buf;
    size_t off = 0;

    while (off < n) {
        ssize_t r = read(fd, p + off, n - off);
        if (r < 0) {
            if (errno == EINTR) continue;
            return YAI_CTL_ERR_READ;
        }
        if (r == 0) return YAI_CTL_ERR_READ; // EOF
        off += (size_t)r;
    }
    return YAI_CTL_OK;
}

static int write_all(int fd, const void *buf, size_t n) {
    const char *p = buf;
    size_t off = 0;

    while (off < n) {
        ssize_t w = write(fd, p + off, n - off);
        if (w < 0) {
            if (errno == EINTR) continue;
            return YAI_CTL_ERR_WRITE;
        }
        if (w == 0) return YAI_CTL_ERR_WRITE;
        off += (size_t)w;
    }
    return YAI_CTL_OK;
}

/* ---------------------------
   Listen at UNIX socket
---------------------------- */
int yai_control_listen_at(const char *path) {
    struct sockaddr_un addr;
    int path_len = 0;
    memset(&addr, 0, sizeof(addr));

    if (!path || !path[0]) return YAI_CTL_ERR_BIND;
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, path, sizeof(addr.sun_path) - 1);
    addr.sun_path[sizeof(addr.sun_path) - 1] = '\0';
    path_len = (int)strlen(addr.sun_path);
    if (path_len <= 0 || path_len >= (int)sizeof(addr.sun_path)) {
        errno = ENAMETOOLONG;
        return YAI_CTL_ERR_BIND;
    }

    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return YAI_CTL_ERR_SOCKET;

    unlink(path); // cleanup previous socket

    {
        socklen_t addr_len = (socklen_t)(offsetof(struct sockaddr_un, sun_path) + path_len + 1);
        if (bind(fd, (struct sockaddr *)&addr, addr_len) < 0) {
            perror("yai: control bind failed");
            close(fd);
            return YAI_CTL_ERR_BIND;
        }
    }

    chmod(path, 0600); // restrict access

    if (listen(fd, YAI_CONTROL_BACKLOG) < 0) {
        perror("yai: control listen failed");
        close(fd);
        return YAI_CTL_ERR_LISTEN;
    }

    return fd;
}

/* ---------------------------
   Read frame (envelope + payload)
---------------------------- */
ssize_t yai_control_read_frame(int fd, yai_rpc_envelope_t *env, void *payload_buf, size_t payload_cap) {
    if (read_all(fd, env, sizeof(*env)) != YAI_CTL_OK) return YAI_CTL_ERR_READ;

    if (env->payload_len > payload_cap) return YAI_CTL_ERR_OVERFLOW;

    if (env->payload_len > 0) {
        if (read_all(fd, payload_buf, env->payload_len) != YAI_CTL_OK) return YAI_CTL_ERR_READ;
    }

    return env->payload_len;
}

/* ---------------------------
   Write frame (envelope + payload)
---------------------------- */
int yai_control_write_frame(int fd, const yai_rpc_envelope_t *env, const void *payload) {
    if (write_all(fd, env, sizeof(*env)) != YAI_CTL_OK) return YAI_CTL_ERR_WRITE;

    if (env->payload_len > 0) {
        if (write_all(fd, payload, env->payload_len) != YAI_CTL_OK) return YAI_CTL_ERR_WRITE;
    }

    return YAI_CTL_OK;
}

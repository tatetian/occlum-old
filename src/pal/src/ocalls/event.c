#define _GNU_SOURCE
#include "ocalls.h"
#include <errno.h>
#include <signal.h>
#include <poll.h>
#include <unistd.h>
#include <sys/eventfd.h>

int occlum_ocall_eventfd(unsigned int initval, int flags) {
    return eventfd(initval, flags);
}

int occlum_ocall_eventfd_poll(int eventfd, struct timespec *timeout) {
    int ret;

    struct pollfd pollfds[1];
    pollfds[0].fd = eventfd;
    pollfds[0].events = POLLIN;
    pollfds[0].revents = 0;

    // We use the ppoll syscall directly instead of the libc wrapper. This
    // is because the syscall version updates the timeout argument to indicate
    // how much time was left (which what we want), while the libc wrapper
    // keeps the timeout argument unchanged.
    ret = raw_ppoll(pollfds, 1, timeout);
    if (ret < 0) {
        return -1;
    }

    char buf[8];
    read(eventfd, buf, 8);
    return 0;
}

void occlum_ocall_eventfd_write_batch(
    int *eventfds,
    size_t num_fds,
    uint64_t val
) {
    for (int fd_i = 0; fd_i < num_fds; fd_i++) {
        write(eventfds[fd_i], &val, sizeof(val));
    }
}

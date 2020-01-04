#include <sys/types.h>
#include <sys/socket.h>
#include <stddef.h>
#include "ocalls.h"

ssize_t occlum_ocall_sendmsg(int sockfd,
                             const void *msg_name,
                             socklen_t msg_namelen,
                             const void *buf,
                             size_t buf_len,
                             const void *msg_control,
                             size_t msg_controllen,
                             int flags)
{
    struct iovec msg_iov = { .iov_base = (void*)buf, .iov_len = buf_len };
    struct iovec* p_msg_iov = buf != NULL ? &msg_iov : NULL;
    size_t msg_iovlen = buf != NULL ? 1 : 0;

    struct msghdr msg = {
        (void*) msg_name,
        msg_namelen,
        p_msg_iov,
        msg_iovlen,
        (void*) msg_control,
        msg_controllen,
        0,
    };
    return sendmsg(sockfd, &msg, flags);
}

ssize_t occlum_ocall_recvmsg(int sockfd,
                             void *msg_name,
                             socklen_t msg_namelen,
                             socklen_t* msg_namelen_recv,
                             void *buf,
                             size_t buf_len,
                             void *msg_control,
                             size_t msg_controllen,
                             size_t* msg_controllen_recv,
                             int* msg_flags_recv,
                             int flags)
{
    struct iovec msg_iov = { .iov_base = buf, .iov_len = buf_len };
    struct iovec* p_msg_iov = buf != NULL ? &msg_iov : NULL;
    size_t msg_iovlen = buf != NULL ? 1 : 0;

    struct msghdr msg = {
        msg_name,
        msg_namelen,
        p_msg_iov,
        msg_iovlen,
        msg_control,
        msg_controllen,
        0,
    };
    ssize_t ret = recvmsg(sockfd, &msg, flags);
    if (ret < 0) return ret;

    *msg_namelen_recv = msg.msg_namelen;
    *msg_controllen_recv = msg.msg_controllen;
    *msg_flags_recv = msg.msg_flags;
    return ret;
}

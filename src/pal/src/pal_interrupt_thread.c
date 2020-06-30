#include <pthread.h>
#include "Enclave_u.h"
#include "pal_enclave.h"
#include "pal_error.h"
#include "pal_interrupt_thread.h"
#include "pal_log.h"
#include "pal_syscall.h"
#include "errno2str.h"

static pthread_t thread;
static int is_running = 0;
static volatile int should_stop = 0;

static void* thread_func(void* _data) {
	sgx_enclave_id_t eid = pal_get_enclave_id();

	while (!should_stop) {
		int ecall_ret;
		sgx_status_t ecall_status = occlum_ecall_broadcast_interrupts(eid, &ecall_ret);
		if (ecall_status != SGX_SUCCESS && ecall_status != SGX_ERROR_OUT_OF_TCS) {
			const char *sgx_err = pal_get_sgx_error_msg(ecall_status);
			PAL_WARN("Failed to do ECall: occlum_ecall_broadcast_interrupts: %s", sgx_err);
			break;
		}

		#define MS 	(1000*1000L) // 1ms = 1,000,000ns
		struct timespec timeout = { .tv_sec = 0, .tv_nsec = 25*MS };
		futex_wait(&should_stop, 0, &timeout);
	}
    
	return NULL;
}

int pal_interrupt_thread_start(void) {
	if (is_running) {
		errno = EEXIST;
		PAL_ERROR("The interrupt thread is already running: %s", errno2str(errno));
		return -1;
	}

	int ret = 0;
	if ((ret = pthread_create(&thread, NULL, thread_func, NULL))) {
		errno = ret;
		PAL_ERROR("Failed to start the interrupt thread: %s", errno2str(errno));
		return -1;
	}

	return 0;
}

int pal_interrupt_thread_stop(void) {
	if (!is_running) {
		errno = ENOENT;
		return -1;
	}
	is_running = 0;

	// Stop the interrupt thread gracefully
	should_stop = 1;
	futex_wake(&should_stop);

	int ret = 0;
	if ((ret = pthread_join(thread)) {
		errno = ret;
		PAL_ERROR("Failed to free the interrupt thread: %s", errno2str(errno));
		return -1;
	}

	return 0;
}
#include "ocalls.h"

int occlum_ocall_tkill(int tid, int signum) {
	int tgid = getpid();
	int ret = tgkill(tgid, tid, signum);
	return ret;
}

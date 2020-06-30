use crate::prelude::*;

use self::sgx::sgx_interrupt_info_t;
use crate::syscall::CpuContext;

mod sgx;


pub fn init() {
    unsafe {
        let status = sgx::sgx_interrupt_init(handle_interrupt);
        assert!(status == sgx_status_t::SGX_SUCCESS);
    }
}

pub fn enable_current_thread() {
    // Interruptible range
    let (addr, size) = {
        let vm = current!().vm().lock().unwrap();
        let range = vm.get_process_range();
        (range.start(), range.size())
    };
    unsafe {
        let status = sgx::sgx_interrupt_enable(addr, size);
        assert!(status == sgx_status_t::SGX_SUCCESS);
    }
}

pub fn disable_current_thread() {
    unsafe {
        let status = sgx::sgx_interrupt_disable();
        assert!(status == sgx_status_t::SGX_SUCCESS);
    }
}

#[no_mangle]
extern "C" fn handle_interrupt(info: *mut sgx_interrupt_info_t) -> i32 {
    extern "C" {
        fn __occlum_syscall_c_abi(num: u32, info: *mut sgx_interrupt_info_t) -> u32;
    }
    unsafe { __occlum_syscall_c_abi(SyscallNum::HandleInterrupt as u32, info) };
    unreachable!();
}

pub fn do_handle_interrupt(
    info: *mut sgx_interrupt_info_t,
    cpu_context: *mut CpuContext,
) -> Result<isize> {
    let info = unsafe { &*info };
    let context = unsafe { &mut *cpu_context };
    // The cpu context is overriden so that it is as if the syscall is called from where the
    // interrupt happened
    *context = CpuContext::from_sgx(&info.cpu_context);
}

pub fn broadcast_interrupts() -> Result<usize> {
    let should_interrupt_thread = |thread: &ThreadRef| -> bool {
        // TODO: check Thread::sig_mask to avoid false positives
        thread.process().is_forced_to_exit()
            || !thread.sig_queues().lock().unwrap().empty()
            || !thread.process().sig_queues().lock().unwrap().empty()
    };

    let num_broadcast_threads = crate::process::table::get_all_threads()
        .iter()
        .filter(should_interrupt_thread)
        .map(|thread| {
            let host_tid = {
                let sched = thread.sched().lock().unwrap();
                match thread.host_tid() {
                    None => return 0;
                    Some(host_tid) => host_tid,
                }
            };
            const signum = 64; // real-time signal 64 is used to notify interrupts
            unsafe {
                let mut retval = 0;
                let status = occlum_ocall_tkill(&mut retval, host_tid, signum);
                assert!(status == sgx_status_t::SGX_SUCCESS);
                if retval ==  0 {
                    1 // increase the success counter
                } else {
                    warn!("occlum_ocall_tkill failed: errno = {:?}", libc::errno());
                    0 // do not increase
                }
            } as usize
        })
        .sum();
    Ok(num_broadcast_threads)
}

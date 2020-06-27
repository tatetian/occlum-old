use crate::prelude::*;

mod sgx_interrupt;

pub fn init() {
    unsafe {
        let status = sgx_interrupt_init(handle_interrupt);
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
        let status = sgx_interrupt_enable(addr, size);
        assert!(status == sgx_status_t::SGX_SUCCESS);
    }
}

pub fn disable_current_thread() {
    unsafe {
        sgx_interrupt_disable();
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
    for thread in process::table::get_all_threads() {
        // TODO: sig_mask should be combined into sig_queues to avoid false positive
        if thread.process().is_forced_exit()
            || !thread.sig_queues().is_empty()
            || !thread.process().sig_queues().is_empty()
        {
            let host_tid = match thread.host_tid() {
				None => continue;
				Some(host_tid) => host_tid,
			};

            let host_tid = thread.host_tid();
            let signum = 64;
            ocall_signal(host_tid, signum);
        }
    }
}

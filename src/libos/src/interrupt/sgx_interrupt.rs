use crate::prelude::*;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct sgx_interrupt_info_t {
    cpu_context: sgx_cpu_context_t,
}

pub type sgx_interrupt_handler_t = extern "C" fn(info: *mut sgx_interrupt_info_t) -> int32_t;

extern "C" {
    pub fn sgx_interrupt_init(handler: sgx_interrupt_handler_t) -> sgx_status_t;
    pub fn sgx_interrupt_enable(code_addr: usize, code_size: usize) -> sgx_status_t;
    pub fn sgx_interrupt_disable() -> sgx_status_t;
};

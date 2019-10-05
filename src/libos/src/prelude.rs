pub use sgx_trts::libc;
pub use sgx_trts::libc::off_t;
pub use sgx_types::*;
use std;

//pub use {elf_helper, errno, file, file_table, fs, mm, process, syscall, vma, };

// TODO: use this one-liner to replace the prelude
pub use std::prelude::v1::*;

pub use std::cell::{Cell, RefCell};
pub use std::marker::{Send, Sync};
pub use std::result::Result;
pub use std::sync::{
    Arc, SgxMutex, SgxMutexGuard, SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard,
};
//pub use std::borrow::BorrowMut;
pub use std::borrow::ToOwned;
pub use std::boxed::Box;
pub use std::cmp::{max, min};
pub use std::cmp::{Ordering, PartialOrd};
pub use std::collections::{HashMap, VecDeque};
pub use std::fmt::{Debug, Display};
pub use std::io::{Read, Seek, SeekFrom, Write};
pub use std::iter::Iterator;
pub use std::rc::Rc;
pub use std::string::{String, ToString};
pub use std::vec::Vec;

macro_rules! debug_trace {
    () => {
        println!("> Line = {}, File = {}", line!(), file!())
    };
}

pub fn align_up(addr: usize, align: usize) -> usize {
    debug_assert!(align != 0 && align.is_power_of_two());
    align_down(addr + (align - 1), align)
}

pub fn align_down(addr: usize, align: usize) -> usize {
    debug_assert!(align != 0 && align.is_power_of_two());
    addr & !(align - 1)
}

pub fn unbox<T>(value: Box<T>) -> T {
    *value
}

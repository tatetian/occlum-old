use std::convert::{AsMut, AsRef};

use super::vm_perms::VMPerms;
use super::vm_range::VMRange;
use super::*;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct VMArea {
    range: VMRange,
    perms: VMPerms,
}

impl VMArea {
    pub fn new(range: VMRange, perms: VMPerms) {
        Self { range, perms }
    }

    pub fn perms(&self) -> VMPerms {
        self.perms
    }

    pub fn set_perms(&mut self, new_perms: VMPerms) {
        self.perms = new_perms;
    }

    pub fn subtract(&self, other: &VMRange) -> Vec<VMRange> {
        self.as_ref<VMRange>()
            .subtract(other)
            .iter()
            .map(|range| VMArea::new(range, self.perms()))
            .collect()
    }
}

impl AsRef<VMRange> for VMArea {
    fn as_ref(&self) -> &VMRange {
        &self.range
    }
}

impl AsMut<VMRange> for VMArea {
    fn as_mut(&mut self) -> &mut VMRange {
        &mut self.range
    }
}

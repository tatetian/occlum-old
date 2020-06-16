use super::*;

bitflags! {
    pub struct VMPerms : u32 {
        const Read        = 0x1;
        const Write       = 0x2;
        const Exec        = 0x4;
        const All         = VMPerms::Read | VMPerms::Write | VMPerms::Exec;
    }
}

impl VMPerms {
    pub fn from_u32(bits: u32) -> Result<VMPerms> {
        let unsupported_bits = bits & (!VMPerms::All);
        if unsupported_bits != 0 {
            warn!(
                "memory perm bits contains unsupported bits ({:?})",
                unsupported_bits
            );
        }

        VMPerms::from_bits_truncate(bits)
    }

    pub fn can_read(&self) -> bool {
        self.contains(VMPerms::Read)
    }

    pub fn can_write(&self) -> bool {
        self.contains(VMPerms::Write)
    }

    pub fn can_execute(&self) -> bool {
        self.contains(VMPerms::Exec)
    }
}

impl Default for VMPerms {
    fn default() -> Self {
        VMPerms::All
    }
}

use super::*;
use rcore_fs_sefs::dev::SefsMac;
use sgx_trts::libc::S_IRUSR;
use std::fmt;

pub struct INodeFile {
    inode: Arc<INode>,
    offset: SgxMutex<usize>,
    access_mode: AccessMode,
    status_flags: SgxRwLock<StatusFlags>,
}

impl File for INodeFile {
    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        if !self.access_mode.readable() {
            return_errno!(EBADF, "File not readable");
        }
        let mut offset = self.offset.lock().unwrap();
        let len = self.inode.read_at(*offset, buf).map_err(|e| errno!(e))?;
        *offset += len;
        Ok(len)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        if !self.access_mode.writable() {
            return_errno!(EBADF, "File not writable");
        }
        let mut offset = self.offset.lock().unwrap();
        if self.status_flags.read().unwrap().always_append() {
            let info = self.inode.metadata()?;
            *offset = info.size;
        }
        let len = self.inode.write_at(*offset, buf)?;
        *offset += len;
        Ok(len)
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        if !self.access_mode.readable() {
            return_errno!(EBADF, "File not readable");
        }
        let len = self.inode.read_at(offset, buf)?;
        Ok(len)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        if !self.access_mode.writable() {
            return_errno!(EBADF, "File not writable");
        }
        let len = self.inode.write_at(offset, buf)?;
        Ok(len)
    }

    fn readv(&self, bufs: &mut [&mut [u8]]) -> Result<usize> {
        if !self.access_mode.readable() {
            return_errno!(EBADF, "File not readable");
        }
        let mut offset = self.offset.lock().unwrap();
        let mut total_len = 0;
        for buf in bufs {
            match self.inode.read_at(*offset, buf) {
                Ok(len) => {
                    total_len += len;
                    *offset += len;
                }
                Err(_) if total_len != 0 => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(total_len)
    }

    fn writev(&self, bufs: &[&[u8]]) -> Result<usize> {
        if !self.access_mode.writable() {
            return_errno!(EBADF, "File not writable");
        }
        let mut offset = self.offset.lock().unwrap();
        if self.status_flags.read().unwrap().always_append() {
            let info = self.inode.metadata()?;
            *offset = info.size;
        }
        let mut total_len = 0;
        for buf in bufs {
            match self.inode.write_at(*offset, buf) {
                Ok(len) => {
                    total_len += len;
                    *offset += len;
                }
                Err(_) if total_len != 0 => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(total_len)
    }

    fn seek(&self, pos: SeekFrom) -> Result<off_t> {
        let mut offset = self.offset.lock().unwrap();
        *offset = match pos {
            SeekFrom::Start(off) => off as usize,
            SeekFrom::End(off) => (self.inode.metadata()?.size as i64 + off) as usize,
            SeekFrom::Current(off) => (*offset as i64 + off) as usize,
        };
        Ok(*offset as i64)
    }

    fn metadata(&self) -> Result<Metadata> {
        let metadata = self.inode.metadata()?;
        Ok(metadata)
    }

    fn set_len(&self, len: u64) -> Result<()> {
        if !self.access_mode.writable() {
            return_errno!(EBADF, "File not writable. Can't set len.");
        }
        self.inode.resize(len as usize)?;
        Ok(())
    }

    fn sync_all(&self) -> Result<()> {
        self.inode.sync_all()?;
        Ok(())
    }

    fn sync_data(&self) -> Result<()> {
        self.inode.sync_data()?;
        Ok(())
    }

    fn read_entry(&self) -> Result<String> {
        if !self.access_mode.readable() {
            return_errno!(EBADF, "File not readable. Can't read entry.");
        }
        let mut offset = self.offset.lock().unwrap();
        let name = self.inode.get_entry(*offset)?;
        *offset += 1;
        Ok(name)
    }

    fn get_access_mode(&self) -> Result<AccessMode> {
        Ok(self.access_mode.clone())
    }

    fn get_status_flags(&self) -> Result<StatusFlags> {
        let status_flags = self.status_flags.read().unwrap();
        Ok(status_flags.clone())
    }

    fn set_status_flags(&self, new_status_flags: StatusFlags) -> Result<()> {
        let mut status_flags = self.status_flags.write().unwrap();
        // Currently, F_SETFL can change only the O_APPEND,
        // O_ASYNC, O_NOATIME, and O_NONBLOCK flags
        let valid_flags_mask = StatusFlags::O_APPEND
            | StatusFlags::O_ASYNC
            | StatusFlags::O_NOATIME
            | StatusFlags::O_NONBLOCK;
        status_flags.remove(valid_flags_mask);
        status_flags.insert(new_status_flags & valid_flags_mask);
        Ok(())
    }

    fn as_any(&self) -> &Any {
        self
    }
}

impl INodeFile {
    pub fn open(inode: Arc<INode>, flags: u32) -> Result<Self> {
        let access_mode = AccessMode::from_u32(flags)?;
        if (access_mode.readable() && !inode.allow_read()?) {
            return_errno!(EBADF, "File not readable");
        }
        if (access_mode.writable() && !inode.allow_write()?) {
            return_errno!(EBADF, "File not writable");
        }
        let status_flags = StatusFlags::from_bits_truncate(flags);
        Ok(INodeFile {
            inode,
            offset: SgxMutex::new(0),
            access_mode,
            status_flags: SgxRwLock::new(status_flags),
        })
    }
}

impl Debug for INodeFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "INodeFile {{ inode: ???, pos: {}, access_mode: {:?}, status_flags: {:#o} }}",
            *self.offset.lock().unwrap(),
            self.access_mode,
            *self.status_flags.read().unwrap()
        )
    }
}

pub trait INodeExt {
    fn read_as_vec(&self) -> Result<Vec<u8>>;
    fn allow_write(&self) -> Result<bool>;
    fn allow_read(&self) -> Result<bool>;
}

impl INodeExt for INode {
    fn read_as_vec(&self) -> Result<Vec<u8>> {
        let size = self.metadata()?.size;
        let mut buf = Vec::with_capacity(size);
        unsafe {
            buf.set_len(size);
        }
        self.read_at(0, buf.as_mut_slice())?;
        Ok(buf)
    }

    fn allow_write(&self) -> Result<bool> {
        let info = self.metadata()?;
        let perms = info.mode as u32;
        let writable = (perms & S_IWUSR) == S_IWUSR;
        Ok(writable)
    }

    fn allow_read(&self) -> Result<bool> {
        let info = self.metadata()?;
        let perms = info.mode as u32;
        let readable = (perms & S_IRUSR) == S_IRUSR;
        Ok(readable)
    }
}

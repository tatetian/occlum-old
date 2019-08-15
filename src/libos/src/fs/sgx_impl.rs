use rcore_fs::dev::TimeProvider;
use rcore_fs::vfs::Timespec;
use rcore_fs_sefs::dev::*;
use std::boxed::Box;
use std::collections::BTreeMap;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sgxfs::{remove, OpenOptions, SgxFile};
use std::sync::{Arc, SgxMutex as Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SgxStorage {
    path: PathBuf,
    integrity_only: bool,
    file_cache: Mutex<BTreeMap<usize, LockedFile>>,
}

impl SgxStorage {
    pub fn new(path: impl AsRef<Path>, integrity_only: bool) -> Self {
        //        assert!(path.as_ref().is_dir());
        SgxStorage {
            path: path.as_ref().to_path_buf(),
            integrity_only: integrity_only,
            file_cache: Mutex::new(BTreeMap::new()),
        }
    }
    /// Get file by `file_id`.
    /// It lookups cache first, if miss, then call `open_fn` to open one,
    /// and add it to cache before return.
    #[cfg(feature = "sgx_file_cache")]
    fn get(&self, file_id: usize, open_fn: impl FnOnce(&Self) -> DevResult<LockedFile>) -> DevResult<LockedFile> {
        // query cache
        let mut caches = self.file_cache.lock().unwrap();
        if let Some(locked_file) = caches.get(&file_id) {
            // hit, return
            return Ok(locked_file.clone());
        }
        // miss, open one
        let locked_file = open_fn(self)?;
        // add to cache
        caches.insert(file_id, locked_file.clone());
        Ok(locked_file)
    }
    /// Get file by `file_id` without cache.
    #[cfg(not(feature = "sgx_file_cache"))]
    fn get(&self, file_id: usize, open_fn: impl FnOnce(&Self) -> DevResult<LockedFile>) -> LockedFile {
        open_fn(self)
    }
}

impl Storage for SgxStorage {
    fn open(&self, file_id: usize) -> DevResult<Box<File>> {
        let locked_file = self.get(file_id, |this| {
            let mut path = this.path.to_path_buf();
            path.push(format!("{}", file_id));
            let options = {
                let mut options = OpenOptions::new();
                options.read(true).update(true);
                options
            };
            let file = {
                let open_res = if !self.integrity_only {
                    options.open(path)
                } else {
                    options.open_integrity_only(path)
                };
                if open_res.is_err() {
                    return Err(DeviceError);
                }
                open_res.unwrap()
            };
            Ok(LockedFile(Arc::new(Mutex::new(file))))
        })?;
        Ok(Box::new(locked_file))
    }

    fn create(&self, file_id: usize) -> DevResult<Box<File>> {
        let locked_file = self.get(file_id, |this| {
            let mut path = this.path.to_path_buf();
            path.push(format!("{}", file_id));
            let options = {
                let mut options = OpenOptions::new();
                options.write(true).update(true);
                options
            };
            let file = {
                let open_res = if !self.integrity_only {
                    options.open(path)
                } else {
                    options.open_integrity_only(path)
                };
                if open_res.is_err() {
                    return Err(DeviceError);
                }
                open_res.unwrap()
            };
            Ok(LockedFile(Arc::new(Mutex::new(file))))
        })?;
        Ok(Box::new(locked_file))
    }

    fn remove(&self, file_id: usize) -> DevResult<()> {
        let mut path = self.path.to_path_buf();
        path.push(format!("{}", file_id));
        remove(path).expect("failed to remove SgxFile");
        // remove from cache
        let mut caches = self.file_cache.lock().unwrap();
        caches.remove(&file_id);
        Ok(())
    }
}

#[derive(Clone)]
pub struct LockedFile(Arc<Mutex<SgxFile>>);

// `sgx_tstd::sgxfs::SgxFile` not impl Send ...
unsafe impl Send for LockedFile {}
unsafe impl Sync for LockedFile {}

impl File for LockedFile {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> DevResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let mut file = self.0.lock().unwrap();
        let offset = offset as u64;
        file.seek(SeekFrom::Start(offset))
            .expect("failed to seek SgxFile");
        let len = file.read(buf).expect("failed to read SgxFile");
        Ok(len)
    }

    fn write_at(&self, buf: &[u8], offset: usize) -> DevResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let mut file = self.0.lock().unwrap();

        // SgxFile do not support seek a position after the end.
        // So check the size and padding zeros if necessary.
        let file_size = file.seek(SeekFrom::End(0)).expect("failed to tell SgxFile") as usize;
        if file_size < offset {
            static ZEROS: [u8; 0x1000] = [0; 0x1000];
            let mut rest_len = offset - file_size;
            while rest_len != 0 {
                let l = rest_len.min(0x1000);
                let len = file.write(&ZEROS[..l]).expect("failed to write SgxFile");
                rest_len -= len;
            }
        }

        let offset = offset as u64;
        file.seek(SeekFrom::Start(offset))
            .expect("failed to seek SgxFile");
        let len = file.write(buf).expect("failed to write SgxFile");
        Ok(len)
    }

    fn set_len(&self, len: usize) -> DevResult<()> {
        // NOTE: do nothing ??
        Ok(())
    }

    fn flush(&self) -> DevResult<()> {
        let mut file = self.0.lock().unwrap();
        file.flush().expect("failed to flush SgxFile");
        Ok(())
    }
}

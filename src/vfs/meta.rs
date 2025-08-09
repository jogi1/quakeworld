#[cfg(target_os = "unix")]
use std::os::unix::fs::MetadataExt;
#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

pub type VfsMetaTimestamp = std::time::SystemTime;
pub fn new_timestamp() -> VfsMetaTimestamp {
    std::time::SystemTime::now()
}

#[derive(Debug, Clone)]
pub struct VfsMetaTime {
    pub created: VfsMetaTimestamp,
    pub modified: VfsMetaTimestamp,
}

impl Default for VfsMetaTime {
    fn default() -> Self {
        Self {
            created: new_timestamp(),
            modified: new_timestamp(),
        }
    }
}

pub type VfsMetaSize = u64;

#[derive(Default, Debug, Clone)]
pub struct VfsMetaData {
    pub time: VfsMetaTime,
    pub size: VfsMetaSize,
}

impl From<std::fs::Metadata> for VfsMetaData {
    fn from(metadata: std::fs::Metadata) -> Self {
        let created = match metadata.created() {
            Ok(c) => c,
            Err(_) => new_timestamp(),
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(_) => new_timestamp(),
        };

        #[cfg(target_os = "windows")]
        let file_size = metadata.file_size();

        #[cfg(not(target_os = "windows"))]
        let file_size = metadata.size();

        Self {
            size: file_size,
            time: VfsMetaTime { created, modified },
        }
    }
}

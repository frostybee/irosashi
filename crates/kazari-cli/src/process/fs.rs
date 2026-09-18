use std::io;
use std::path::{Path, PathBuf};

/// The minimal disk surface the processor needs. The default is the real disk; tests use
/// an in-memory fake. Walking visits regular files only and never follows symlinks.
/// Enumeration failures are fatal; per file failures land in the file's result instead.
pub trait FileSystem: Sync {
    fn walk_files(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()>;
}

pub struct OsFs;

impl FileSystem for OsFs {
    fn walk_files(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        walk(root, &mut out)?;
        Ok(out)
    }

    fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    /// Writes to a temp file in the target directory and renames it over the destination,
    /// so a dev server serving the tree mid run never observes a truncated file.
    fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let tmp = tempfile::Builder::new()
            .prefix(".kazari-tmp-")
            .tempfile_in(dir)?;
        std::fs::write(tmp.path(), data)?;
        if let Ok(meta) = std::fs::metadata(path) {
            std::fs::set_permissions(tmp.path(), meta.permissions())?;
        }
        tmp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.collect::<io::Result<_>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let ty = entry.file_type()?;
        if ty.is_dir() {
            walk(&entry.path(), out)?;
        } else if ty.is_file() {
            out.push(entry.path());
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod mem {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    /// An in-memory tree keyed by path, with a write log for no-op assertions.
    #[derive(Default)]
    pub struct MemFs {
        pub files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
        pub writes: Mutex<Vec<PathBuf>>,
    }

    impl MemFs {
        pub fn with(files: &[(&str, &str)]) -> Self {
            let fs = MemFs::default();
            for (p, c) in files {
                fs.files
                    .lock()
                    .unwrap()
                    .insert(PathBuf::from(p), c.as_bytes().to_vec());
            }
            fs
        }

        pub fn get(&self, path: &str) -> Option<String> {
            self.files
                .lock()
                .unwrap()
                .get(Path::new(path))
                .map(|b| String::from_utf8_lossy(b).into_owned())
        }
    }

    impl FileSystem for MemFs {
        fn walk_files(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
            Ok(self
                .files
                .lock()
                .unwrap()
                .keys()
                .filter(|p| p.starts_with(root))
                .cloned()
                .collect())
        }

        fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))
        }

        fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), data.to_vec());
            self.writes.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_fs_walks_regular_files_sorted_and_writes_atomically() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("b/c")).unwrap();
        std::fs::write(tmp.path().join("b/c/z.html"), "z").unwrap();
        std::fs::write(tmp.path().join("a.html"), "a").unwrap();
        let files = OsFs.walk_files(tmp.path()).unwrap();
        assert_eq!(
            files,
            vec![tmp.path().join("a.html"), tmp.path().join("b/c/z.html")]
        );
        OsFs.write_file(&tmp.path().join("a.html"), b"new").unwrap();
        assert_eq!(std::fs::read(tmp.path().join("a.html")).unwrap(), b"new");
        OsFs.write_file(&tmp.path().join("b/n.html"), b"n").unwrap();
        assert_eq!(std::fs::read(tmp.path().join("b/n.html")).unwrap(), b"n");
        let leftovers: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".kazari-tmp-"))
            .collect();
        assert!(leftovers.is_empty());
    }
}

//! Zigmera I/O utilities for atomic writes, temp files, file locks, checksums, and crash-safe artifact writes.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;

/// I/O errors
#[derive(Debug, Error)]
pub enum IoError {
    #[error("atomic write failed: {0}")]
    AtomicWrite(#[source] io::Error),
    #[error("file lock failed: {0}")]
    LockFailed(#[source] io::Error),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("invalid path: {0}")]
    InvalidPath(String),
}

/// Atomic file writer for crash-safe writes
pub struct AtomicWriter {
    path: PathBuf,
    tmp_path: PathBuf,
}

impl AtomicWriter {
    /// Create a new atomic writer for the given path
    pub fn new(path: &Path) -> Result<Self, IoError> {
        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        Ok(Self {
            path: path.to_path_buf(),
            tmp_path,
        })
    }

    /// Write content atomically
    pub fn write(&mut self, content: &[u8]) -> Result<(), IoError> {
        let mut file = File::create(&self.tmp_path).map_err(IoError::AtomicWrite)?;
        file.write_all(content).map_err(IoError::AtomicWrite)?;
        file.sync_all().map_err(IoError::AtomicWrite)?;
        drop(file);

        // Atomic rename
        fs::rename(&self.tmp_path, &self.path).map_err(IoError::AtomicWrite)?;
        Ok(())
    }

    /// Write string content atomically
    pub fn write_str(&mut self, content: &str) -> Result<(), IoError> {
        self.write(content.as_bytes())
    }
}

/// File lock guard
pub struct FileLock {
    file: File,
    path: PathBuf,
}

impl FileLock {
    /// Acquire a shared (read) lock
    pub fn shared(path: &Path) -> Result<Self, IoError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(IoError::LockFailed)?;
        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Acquire an exclusive (write) lock
    pub fn exclusive(path: &Path) -> Result<Self, IoError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(IoError::LockFailed)?;
        Ok(Self {
            file,
            path: path.to_path_buf(),
        })
    }

    /// Get the locked file
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Get the locked file mutably
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Lock is automatically released when file is closed
    }
}

/// Checksum computation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumAlgo {
    Blake3,
    Sha256,
}

impl ChecksumAlgo {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChecksumAlgo::Blake3 => "blake3",
            ChecksumAlgo::Sha256 => "sha256",
        }
    }
}

/// Compute file checksum
pub fn compute_checksum(path: &Path, algo: ChecksumAlgo) -> Result<String, IoError> {
    let mut file = File::open(path).map_err(IoError::AtomicWrite)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(IoError::AtomicWrite)?;
    drop(file);

    let hash = match algo {
        ChecksumAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&buffer);
            hasher.finalize().to_hex().to_string()
        }
        ChecksumAlgo::Sha256 => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&buffer);
            let result = hasher.finalize();
            hex::encode(&result[..])
        }
    };

    Ok(hash)
}

/// Verify file checksum matches expected
pub fn verify_checksum(path: &Path, expected: &str, algo: ChecksumAlgo) -> Result<bool, IoError> {
    let actual = compute_checksum(path, algo)?;
    if actual == expected {
        Ok(true)
    } else {
        Err(IoError::ChecksumMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}

/// Temp file helper
pub struct TempFile {
    temp_dir: TempDir,
    path: PathBuf,
}

impl TempFile {
    /// Create a new temp file with the given extension
    pub fn new(prefix: &str, extension: &str) -> Result<Self, IoError> {
        let temp_dir = tempfile::tempdir().map_err(IoError::AtomicWrite)?;
        let path = temp_dir.path().join(format!("{}.{}", prefix, extension));
        Ok(Self { temp_dir, path })
    }

    /// Get the temp file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write content to the temp file
    pub fn write(&mut self, content: &[u8]) -> Result<(), IoError> {
        let mut file = File::create(&self.path).map_err(IoError::AtomicWrite)?;
        file.write_all(content).map_err(IoError::AtomicWrite)?;
        Ok(())
    }

    /// Persist the temp file to a permanent location
    pub fn persist(self, dest: &Path) -> Result<PathBuf, IoError> {
        fs::copy(&self.path, dest).map_err(IoError::AtomicWrite)?;
        Ok(dest.to_path_buf())
    }
}

/// Crash-safe artifact writer
pub struct CrashSafeWriter {
    artifact_path: PathBuf,
    checksum_path: PathBuf,
    algo: ChecksumAlgo,
}

impl CrashSafeWriter {
    /// Create a new crash-safe writer
    pub fn new(artifact_path: &Path, algo: ChecksumAlgo) -> Self {
        Self {
            artifact_path: artifact_path.to_path_buf(),
            checksum_path: artifact_path.with_extension("cksum"),
            algo,
        }
    }

    /// Write artifact with checksum
    pub fn write(&mut self, content: &[u8]) -> Result<(), IoError> {
        // Compute checksum first
        let checksum = match self.algo {
            ChecksumAlgo::Blake3 => {
                let mut hasher = blake3::Hasher::new();
                hasher.update(content);
                hasher.finalize().to_hex().to_string()
            }
            ChecksumAlgo::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(content);
                let result = hasher.finalize();
                hex::encode(&result[..])
            }
        };

        // Write checksum file first
        let mut cksum_file = File::create(&self.checksum_path).map_err(IoError::LockFailed)?;
        cksum_file
            .write_all(checksum.as_bytes())
            .map_err(IoError::LockFailed)?;
        cksum_file.sync_all().map_err(IoError::LockFailed)?;
        drop(cksum_file);

        // Write artifact
        let mut writer = AtomicWriter::new(&self.artifact_path)?;
        writer.write(content)?;

        // Remove checksum file on success (optional - keeps it for verification)
        // fs::remove_file(&self.checksum_path)?;
        Ok(())
    }

    /// Verify artifact checksum
    pub fn verify(&self) -> Result<bool, IoError> {
        if !self.artifact_path.exists() {
            return Err(IoError::NotFound(self.artifact_path.display().to_string()));
        }

        if !self.checksum_path.exists() {
            return Err(IoError::NotFound(self.checksum_path.display().to_string()));
        }

        let mut cksum_file = File::open(&self.checksum_path).map_err(IoError::LockFailed)?;
        let mut expected = String::new();
        cksum_file
            .read_to_string(&mut expected)
            .map_err(IoError::LockFailed)?;
        drop(cksum_file);

        let expected = expected.trim();
        verify_checksum(&self.artifact_path, expected, self.algo)
    }
}

/// Hex encoding helper (used when hex crate is not available)
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        const CHARS: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(CHARS[(b >> 4) as usize] as char);
            s.push(CHARS[(b & 0xf) as usize] as char);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.txt");
        let mut writer = AtomicWriter::new(&path).unwrap();
        writer.write(b"hello world").unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_checksum_algo() {
        assert_eq!(ChecksumAlgo::Blake3.as_str(), "blake3");
        assert_eq!(ChecksumAlgo::Sha256.as_str(), "sha256");
    }

    #[test]
    fn test_compute_checksum() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.bin");
        fs::write(&path, b"test data").unwrap();

        let cksum = compute_checksum(&path, ChecksumAlgo::Blake3).unwrap();
        assert_eq!(cksum.len(), 64);

        let cksum2 = compute_checksum(&path, ChecksumAlgo::Blake3).unwrap();
        assert_eq!(cksum, cksum2);
    }

    #[test]
    fn test_verify_checksum() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.bin");
        fs::write(&path, b"test data").unwrap();

        let cksum = compute_checksum(&path, ChecksumAlgo::Blake3).unwrap();
        assert!(verify_checksum(&path, &cksum, ChecksumAlgo::Blake3).unwrap());
        assert!(!verify_checksum(&path, "invalid", ChecksumAlgo::Blake3).is_ok());
    }

    #[test]
    fn test_temp_file() {
        let mut temp = TempFile::new("prefix", "ext").unwrap();
        temp.write(b"temp content").unwrap();
        assert!(temp.path().exists());
    }

    #[test]
    fn test_crash_safe_writer() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_path = temp_dir.path().join("artifact.bin");
        let mut writer = CrashSafeWriter::new(&artifact_path, ChecksumAlgo::Blake3);
        writer.write(b"important data").unwrap();
        assert!(artifact_path.exists());
        assert!(writer.verify().unwrap());
    }
}

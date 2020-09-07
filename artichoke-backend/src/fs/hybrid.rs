use std::borrow::Cow;
use std::io;
use std::path::Path;

use crate::fs::{ExtensionHook, Memory, Native, MEMORY_FILESYSTEM_MOUNT_POINT};

#[derive(Default, Debug)]
pub struct Hybrid {
    memory: Memory,
    native: Native,
}

impl Hybrid {
    /// Create a new hybrid virtual filesystem.
    ///
    /// This filesystem allows access to the host filesystem with an in-memory
    /// filesystem mounted at [`MEMORY_FILESYSTEM_MOUNT_POINT`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether `path` points to a file in the virtual filesystem.
    ///
    /// This API is infallible and will return `false` for non-existent paths.
    pub fn is_file(&self, path: &Path) -> bool {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.is_file(path)
        } else {
            self.native.is_file(path)
        }
    }

    /// Read file contents for the file at `path`.
    ///
    /// Returns a byte slice of complete file contents. If `path` is relative,
    /// it is absolutized relative to the current working directory of the
    /// virtual file system.
    ///
    /// # Errors
    ///
    /// If `path` does not exist, an [`io::Error`] with error kind
    /// [`io::ErrorKind::NotFound`] is returned.
    pub fn read_file(&self, path: &Path) -> io::Result<Cow<'_, [u8]>> {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.read_file(path)
        } else {
            self.native.read_file(path)
        }
    }

    /// Write file contents into the virtual file system at `path`.
    ///
    /// Writes the full file contents. If any file contents already exist at
    /// `path`, they are replaced. Extension hooks are preserved.
    ///
    /// # Errors
    ///
    /// If access to the [`Native`] filesystem returns an error, the error is
    /// returned. See [`Native::write_file`].
    pub fn write_file(&mut self, path: &Path, buf: Cow<'static, [u8]>) -> io::Result<()> {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.write_file(path, buf)
        } else {
            self.native.write_file(path, buf)
        }
    }

    /// Retrieve an extension hook for the file at `path`.
    ///
    /// This API is infallible and will return `None` for non-existent paths.
    pub fn get_extension(&self, path: &Path) -> Option<ExtensionHook> {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.get_extension(path)
        } else {
            None
        }
    }

    /// Write extension hook into the virtual file system at `path`.
    ///
    /// If any extension hooks already exist at `path`, they are replaced. File
    /// contents are preserved.
    ///
    /// This function writes all extensions to the virtual filesystem. If the
    /// given path does not map to the virtual filesystem, the extension is
    /// unreachable.
    ///
    /// # Errors
    ///
    /// If the given path does not resolve to the virtual filesystem, an error
    /// is returned.
    pub fn register_extension(&mut self, path: &Path, extension: ExtensionHook) -> io::Result<()> {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.register_extension(path, extension)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Native filesystem does not support extensions",
            ))
        }
    }

    /// Check whether a file at `path` has been required already.
    ///
    /// This API is infallible and will return `false` for non-existent paths.
    pub fn is_required(&self, path: &Path) -> bool {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.is_required(path)
        } else {
            self.native.is_required(path)
        }
    }

    /// Mark a source at `path` as required on the interpreter.
    ///
    /// This metadata is used by `Kernel#require` and friends to enforce that
    /// Ruby sources are only loaded into the interpreter once to limit side
    /// effects.
    ///
    /// # Errors
    ///
    /// If `path` does not exist, an [`io::Error`] with error kind
    /// [`io::ErrorKind::NotFound`] is returned.
    pub fn mark_required(&mut self, path: &Path) -> io::Result<()> {
        if let Ok(path) = path.strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT) {
            self.memory.mark_required(path)
        } else {
            self.native.mark_required(path)
        }
    }
}

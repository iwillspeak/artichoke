use std::borrow::Cow;
use std::collections::hash_map::Entry as HashEntry;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::fs::{
    absolutize_relative_to, ExtensionHook, MEMORY_FILESYSTEM_MOUNT_POINT, RUBY_LOAD_PATH,
};

const CODE_DEFAULT_CONTENTS: &[u8] = b"# virtual source file";

#[derive(Clone, Copy)]
pub struct Extension {
    hook: ExtensionHook,
}

impl From<ExtensionHook> for Extension {
    fn from(hook: ExtensionHook) -> Self {
        Self { hook }
    }
}

impl Extension {
    pub fn new(hook: ExtensionHook) -> Self {
        Self { hook }
    }
}

impl fmt::Debug for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extension")
            .field("hook", &"fn(&mut Artichoke) -> Result<(), Exception>")
            .finish()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Code {
    content: Cow<'static, [u8]>,
}

impl Default for Code {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Code> for Cow<'static, [u8]> {
    fn from(code: Code) -> Self {
        code.into_inner()
    }
}

impl From<Vec<u8>> for Code {
    fn from(content: Vec<u8>) -> Self {
        let content = content.into();
        Self { content }
    }
}

impl From<&'static [u8]> for Code {
    fn from(content: &'static [u8]) -> Self {
        let content = content.into();
        Self { content }
    }
}

impl From<Cow<'static, [u8]>> for Code {
    fn from(content: Cow<'static, [u8]>) -> Self {
        Self { content }
    }
}

impl From<String> for Code {
    fn from(content: String) -> Self {
        let content = content.into_bytes().into();
        Self { content }
    }
}

impl From<&'static str> for Code {
    fn from(content: &'static str) -> Self {
        let content = content.as_bytes().into();
        Self { content }
    }
}

impl From<Cow<'static, str>> for Code {
    fn from(content: Cow<'static, str>) -> Self {
        match content {
            Cow::Borrowed(content) => Self::from(content.as_bytes()),
            Cow::Owned(content) => Self::from(content.into_bytes()),
        }
    }
}

impl Code {
    #[must_use]
    pub const fn new() -> Self {
        let content = Cow::Borrowed(CODE_DEFAULT_CONTENTS);
        Self { content }
    }

    #[must_use]
    pub fn into_inner(self) -> Cow<'static, [u8]> {
        self.content
    }
}

#[derive(Default, Debug)]
pub struct Entry {
    code: Option<Code>,
    extension: Option<Extension>,
}

impl From<Code> for Entry {
    fn from(code: Code) -> Self {
        let mut entry = Self::new();
        entry.code = Some(code);
        entry
    }
}

impl From<Vec<u8>> for Entry {
    fn from(content: Vec<u8>) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<&'static [u8]> for Entry {
    fn from(content: &'static [u8]) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<Cow<'static, [u8]>> for Entry {
    fn from(content: Cow<'static, [u8]>) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<String> for Entry {
    fn from(content: String) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<&'static str> for Entry {
    fn from(content: &'static str) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<Cow<'static, str>> for Entry {
    fn from(content: Cow<'static, str>) -> Self {
        let mut entry = Self::new();
        entry.code = Some(content.into());
        entry
    }
}

impl From<ExtensionHook> for Entry {
    fn from(hook: ExtensionHook) -> Self {
        let mut entry = Self::new();
        entry.extension = Some(hook.into());
        entry
    }
}

impl Entry {
    const fn new() -> Self {
        Self {
            code: None,
            extension: None,
        }
    }

    pub fn replace_content<T>(&mut self, content: T)
    where
        T: Into<Cow<'static, [u8]>>,
    {
        self.code.replace(Code::from(content.into()));
    }

    pub fn set_extension(&mut self, hook: ExtensionHook) {
        self.extension.replace(Extension::new(hook));
    }

    pub fn extension(&self) -> Option<ExtensionHook> {
        self.extension.as_ref().map(|ext| ext.hook)
    }
}

/// Virtual filesystem for sources, extensions, and require metadata.
///
/// `Memory` is a [`HashMap`] from paths to an entry struct that contains:
///
/// - A bit for whether the path that points to the entry has been required
///   before.
/// - Optional binary content representing Ruby source code.
/// - Optional hook to a Rust function to be executed on `require` (similar to a
///   MRI C extension rubygem).
///
/// Sources in `Memory` are only writable via the
/// [`LoadSources`](crate::core::LoadSources) trait. Sources can only be
/// completely replaced.
///
/// These APIs are consumed primarily by the `Kernel::require` implementation in
/// [`extn::core::kernel::require`](crate::extn::core::kernel::require).
#[derive(Debug)]
pub struct Memory {
    fs: HashMap<PathBuf, Entry>,
    loaded_features: HashSet<PathBuf>,
    cwd: PathBuf,
}

impl Default for Memory {
    /// Virtual filesystem with current working directory set to
    /// [`RUBY_LOAD_PATH`].
    fn default() -> Self {
        let cwd = Path::new(RUBY_LOAD_PATH)
            .strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT)
            .unwrap_or_else(|_| Path::new(RUBY_LOAD_PATH))
            .to_path_buf();
        Self {
            fs: HashMap::default(),
            loaded_features: HashSet::default(),
            cwd,
        }
    }
}

impl Memory {
    /// Create a new in memory virtual filesystem.
    ///
    /// Sets the current working directory of the VFS to [`RUBY_LOAD_PATH`] for
    /// storing Ruby source files. This path is searched by
    /// [`Kernel::require`, `Kernel::require_relative`, and `Kernel::load`].
    ///
    /// [`Kernel::require`, `Kernel::require_relative`, and `Kernel::load`]: crate::extn::core::kernel::require
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new in memory virtual filesystem with the given working
    /// directory.
    #[must_use]
    pub fn with_working_directory<T>(cwd: T) -> Self
    where
        T: Into<PathBuf>,
    {
        let cwd = cwd.into();
        let cwd = cwd
            .strip_prefix(MEMORY_FILESYSTEM_MOUNT_POINT)
            .map(Path::to_path_buf)
            .unwrap_or(cwd);
        Self {
            fs: HashMap::default(),
            loaded_features: HashSet::default(),
            cwd,
        }
    }

    /// Check whether `path` points to a file in the virtual filesystem.
    ///
    /// This API is infallible and will return `false` for non-existent paths.
    #[must_use]
    pub fn is_file(&self, path: &Path) -> bool {
        let path = absolutize_relative_to(path, &self.cwd);
        self.fs.contains_key(&path)
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
        let path = absolutize_relative_to(path, &self.cwd);
        if let Some(ref entry) = self.fs.get(&path) {
            if let Some(ref code) = entry.code {
                match code.content {
                    Cow::Borrowed(content) => Ok(content.into()),
                    Cow::Owned(ref content) => Ok(content.clone().into()),
                }
            } else {
                Ok(Code::new().into())
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "file not found in virtual file system",
            ))
        }
    }

    /// Write file contents into the virtual file system at `path`.
    ///
    /// Writes the full file contents. If any file contents already exist at
    /// `path`, they are replaced. Extension hooks are preserved.
    ///
    /// # Errors
    ///
    /// This API is currently infallible but returns [`io::Result`] to reserve
    /// the ability to return errors in the future.
    pub fn write_file(&mut self, path: &Path, buf: Cow<'static, [u8]>) -> io::Result<()> {
        let path = absolutize_relative_to(path, &self.cwd);
        match self.fs.entry(path) {
            HashEntry::Occupied(mut entry) => {
                entry.get_mut().replace_content(buf);
            }
            HashEntry::Vacant(entry) => {
                entry.insert(Entry::from(buf));
            }
        }
        Ok(())
    }

    /// Retrieve an extension hook for the file at `path`.
    ///
    /// This API is infallible and will return `None` for non-existent paths.
    #[must_use]
    pub fn get_extension(&self, path: &Path) -> Option<ExtensionHook> {
        let path = absolutize_relative_to(path, &self.cwd);
        if let Some(entry) = self.fs.get(&path) {
            entry.extension()
        } else {
            None
        }
    }

    /// Write extension hook into the virtual file system at `path`.
    ///
    /// If any extension hooks already exist at `path`, they are replaced. File
    /// contents are preserved.
    ///
    /// # Errors
    ///
    /// This API is currently infallible but returns [`io::Result`] to reserve
    /// the ability to return errors in the future.
    pub fn register_extension(&mut self, path: &Path, extension: ExtensionHook) -> io::Result<()> {
        let path = absolutize_relative_to(path, &self.cwd);
        match self.fs.entry(path) {
            HashEntry::Occupied(mut entry) => {
                entry.get_mut().set_extension(extension);
            }
            HashEntry::Vacant(entry) => {
                entry.insert(Entry::from(extension));
            }
        }
        Ok(())
    }

    /// Check whether a file at `path` has been required already.
    ///
    /// This API is infallible and will return `false` for non-existent paths.
    #[must_use]
    pub fn is_required(&self, path: &Path) -> bool {
        let path = absolutize_relative_to(path, &self.cwd);
        self.loaded_features.contains(&path)
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
        let path = absolutize_relative_to(path, &self.cwd);
        self.loaded_features.insert(path);
        Ok(())
    }
}

#[cfg(test)]
mod hook_prototype_tests {
    use crate::test::prelude::*;

    use super::Extension;

    struct TestFile;

    impl File for TestFile {
        type Artichoke = Artichoke;
        type Error = Error;

        fn require(_interp: &mut Artichoke) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn prototype() {
        // must compile
        let _ = Extension::new(TestFile::require);
    }
}

//! Virtual filesystem.
//!
//! Artichoke proxies all filesystem access through a virtual filesystem. The
//! filesystem can store Ruby sources and [extension hooks](ExtensionHook) in
//! memory and will support proxying to the host filesystem for reads and
//! writes.
//!
//! Artichoke uses the virtual filesystem to track metadata about loaded
//! features.
//!
//! Artichoke has several virtual filesystem implementations. Only some of them
//! support reading from the system fs.

use std::path::{Component, Path, PathBuf};

use crate::error::Error;
use crate::Artichoke;

mod hybrid;
mod memory;
mod native;

pub use hybrid::Hybrid;
pub use memory::Memory;
pub use native::Native;

/// Directory at which the [in-memory filesystem](Memory) is mounted.
///
/// [`Hybrid`] filesystems mount the `Memory` filesystem at this path.
/// `RUBY_LOAD_PATH` is found within this path prefix.
pub const MEMORY_FILESYSTEM_MOUNT_POINT: &str = "/artichoke/virtual_root";

/// Directory at which Ruby sources and extensions are stored in the virtual
/// filesystem.
///
/// `RUBY_LOAD_PATH` is the default current working directory for
/// [`Memory`] filesystems.
///
/// [`Hybrid`] filesystems locate the this path on a `Memory` filesystem below
/// [`MEMORY_FILESYSTEM_MOUNT_POINT`].
pub const RUBY_LOAD_PATH: &str = "/artichoke/virtual_root/src/lib";

/// Function type for extension hooks stored in the virtual filesystem.
///
/// This signature is equivalent to the signature for [`File::require`] as
/// defined by the `artichoke-backend` implementation of [`LoadSources`].
///
/// [`File::require`]: artichoke_core::file::File::require
/// [`LoadSources`]: crate::core::LoadSources
pub type ExtensionHook = fn(&mut Artichoke) -> Result<(), Error>;

#[cfg(all(feature = "native-filesystem-access", not(any(test, doctest))))]
pub type Adapter = Hybrid;
#[cfg(any(not(feature = "native-filesystem-access"), test, doctest))]
pub type Adapter = Memory;

fn absolutize_relative_to<T, U>(path: T, cwd: U) -> PathBuf
where
    T: AsRef<Path>,
    U: AsRef<Path>,
{
    let mut iter = path.as_ref().components().peekable();
    let hint = iter.size_hint();
    let (mut components, cwd_is_relative) = if let Some(Component::RootDir) = iter.peek() {
        (Vec::with_capacity(hint.1.unwrap_or(hint.0)), false)
    } else {
        let mut components = cwd
            .as_ref()
            .components()
            .map(Component::as_os_str)
            .collect::<Vec<_>>();
        components.reserve(hint.1.unwrap_or(hint.0));
        (components, cwd.as_ref().is_relative())
    };
    for component in iter {
        match component {
            Component::CurDir => {}
            Component::ParentDir if cwd_is_relative => {
                components.pop();
            }
            Component::ParentDir => {
                components.pop();
                if components.is_empty() {
                    components.push(Component::RootDir.as_os_str());
                }
            }
            c => {
                components.push(c.as_os_str());
            }
        }
    }
    components.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::absolutize_relative_to;

    #[test]
    fn absolutize_absolute_path() {
        let path = Path::new("/foo/bar");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), path);
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), path);
    }

    #[test]
    fn absolutize_absolute_path_dedot_current_dir() {
        let path = Path::new("/././foo/./bar/./././.");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo/bar"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo/bar"));
    }

    #[test]
    fn absolutize_absolute_path_dedot_parent_dir() {
        let path = Path::new("/foo/bar/..");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/foo"));

        let path = Path::new("/foo/../../../../bar/../../../");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/"));

        let path = Path::new("/foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        assert_eq!(absolutize_relative_to(&path, cwd), Path::new("/boom/baz"));
    }

    #[test]
    fn absolutize_relative_path() {
        let path = Path::new("foo/bar");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(
            absolutize_relative_to(&path, cwd),
            Path::new("/home/artichoke/foo/bar")
        );
        let cwd = Path::new("relative/path");
        assert_eq!(
            absolutize_relative_to(&path, cwd),
            Path::new("relative/path/foo/bar")
        );
    }

    #[test]
    fn absolutize_relative_path_dedot_current_dir() {
        let path = Path::new("././././foo/./bar/./././.");
        let cwd = Path::new("/home/artichoke");
        assert_eq!(
            absolutize_relative_to(&path, cwd),
            Path::new("/home/artichoke/foo/bar")
        );
        let cwd = Path::new("relative/path");
        assert_eq!(
            absolutize_relative_to(&path, cwd),
            Path::new("relative/path/foo/bar")
        );
    }

    #[test]
    #[cfg(unix)]
    fn absolutize_relative_path_dedot_parent_dir_unix() {
        let path = Path::new("foo/bar/..");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/home/artichoke/foo"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new("foo/../../../../bar/../../../");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new("foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("/home/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }

    #[test]
    #[cfg(windows)]
    fn absolutize_relative_path_dedot_parent_dir_windows_forward_slash() {
        let path = Path::new("foo/bar/..");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("C:/Users/artichoke/foo"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new("foo/../../../../bar/../../../");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new("foo/../../../../bar/../../../boom/baz");
        let cwd = Path::new("C:/Users/artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new("relative/path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }

    #[test]
    #[cfg(windows)]
    fn absolutize_relative_path_dedot_parent_dir_windows_backward_slash() {
        let path = Path::new(r"foo\bar\..");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("C:/Users/artichoke/foo"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("relative/path/foo"));

        let path = Path::new(r"foo\..\..\..\..\bar\..\..\..\");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new(""));

        let path = Path::new(r"foo\..\..\..\..\bar\..\..\..\boom\baz");
        let cwd = Path::new(r"C:\Users\artichoke");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("/boom/baz"));
        let cwd = Path::new(r"relative\path");
        let absolute = absolutize_relative_to(&path, cwd);
        assert_eq!(absolute, Path::new("boom/baz"));
    }
}

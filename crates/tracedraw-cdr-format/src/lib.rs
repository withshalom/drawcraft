//! tracedraw-cdr-format: the structure of `.cdr` files, independent of any
//! document model.
//!
//! What is known about the format comes from public format notes and from
//! inspecting files we own, never from any vendor's code or from another
//! implementation (see `CLEANROOM.md` at the root of the repository).
//!
//! The crate holds the layers below the document:
//! - [`container`]: detect a bare RIFF file or a ZIP package, hand back the
//!   RIFF stream and the external data streams.
//! - [`riff`]: walk the chunk tree without interpreting it, inflate the
//!   compressed lists, follow the redirect records of version 16 and later.
//! - [`loda`]: the argument table of the record describing an object.
//! - [`raw`]: the records a reader leaves uninterpreted, kept as bytes
//!   shared with the tree, by the object they belong to.
//! - [`eot`]: the Embedded OpenType structure around an embedded font.
//! - [`ole`]: compound files holding embedded objects and their pictures.
//!
//! Turning chunks into a drawing is the job of the crate above this one
//! (`tracedraw-cdr`). Nothing here depends on the rest of the workspace.
#![forbid(unsafe_code)]
// Borrowed from TraceDraw and kept close to it so its fixes can be brought over. The no-panic
// lints (unwrap, expect, panic and the rest) apply here as everywhere; these three are style only.
#![allow(
    clippy::cognitive_complexity,
    clippy::too_many_arguments,
    clippy::wildcard_imports
)]

pub mod container;
pub mod eot;
pub mod loda;
pub mod ole;
pub mod raw;
pub mod riff;

pub use container::{Container, ContainerKind, Version};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a CDR file")]
    NotCdr,
    #[error("not a CMX file")]
    NotCmx,
    #[error("unsupported container: {0}")]
    UnsupportedContainer(String),
    #[error("truncated data at offset {0}")]
    Truncated(usize),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Fonts for tests, from the shared font collection (`CRAFT_FONTS_DIR`, see VectorCraft's
/// `docs/development.md` › Fonts): no font file is kept in this repository. TraceDraw's tests use
/// its bundled Selawik faces (SIL Open Font License); a test whose face is missing says so and
/// skips.
#[cfg(any(test, feature = "test-support"))]
pub mod test_fonts {
    use std::path::{Path, PathBuf};

    /// The bytes of the font file named `file` (like `Selawik-Regular.ttf`) under
    /// `CRAFT_FONTS_DIR`, or `None` (with a note on standard error) when it isn't there.
    pub fn selawik(file: &str) -> Option<Vec<u8>> {
        let found = std::env::var_os("CRAFT_FONTS_DIR")
            .filter(|d| !d.is_empty())
            .and_then(|d| find(Path::new(&d), file, 0));
        let bytes = found.and_then(|p| std::fs::read(p).ok());
        if bytes.is_none() {
            eprintln!("skipped: {file} is not in CRAFT_FONTS_DIR");
        }
        bytes
    }

    fn find(dir: &Path, file: &str, depth: usize) -> Option<PathBuf> {
        if depth > 6 {
            return None;
        }
        let entries = std::fs::read_dir(dir).ok()?;
        let mut dirs = Vec::new();
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else if p.file_name().is_some_and(|n| n == file) {
                return Some(p);
            }
        }
        dirs.into_iter().find_map(|d| find(&d, file, depth + 1))
    }
}

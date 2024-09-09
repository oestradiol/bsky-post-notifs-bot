use std::path::{Path, PathBuf};

/// This function takes a path and returns the canonicalized path.
/// If the path does not exist, it will attempt to find the closest existing ancestor.
/// Then, it will join the relative path to the canonicalized ancestor.
///
/// # Returns
/// The canonicalized path, even if it doesn't exist.
pub fn canonicalize_unexistent(s: &Path) -> Option<PathBuf> {
  for p in s.ancestors() {
    if let Some(path) = (|| {
      let canonical = p.canonicalize().ok()?;
      let stripped = s.strip_prefix(p).ok()?;
      Some::<PathBuf>(canonical.join(stripped))
    })() {
      return Some(path);
    };
  }
  None
}

use std::path::{Path, PathBuf};

#[must_use]
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

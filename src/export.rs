//! Writes wallpapers to disk as PNG. Files land in a predictable output
//! directory, named from the configuration, never overwriting an existing file.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Config;
use crate::render::render_png;
use crate::util::slug;

/// Where exported wallpapers are written. Override with `HYPSO_OUTPUT_DIR`.
pub fn output_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("HYPSO_OUTPUT_DIR") {
        return PathBuf::from(custom);
    }
    dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hypso-wallpapers")
}

/// A non-existing path in `dir` for `stem.ext`, suffixing `-N` on collision.
fn unique_path(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let mut candidate = dir.join(format!("{stem}.{extension}"));
    let mut counter = 2;
    while candidate.exists() {
        candidate = dir.join(format!("{stem}-{counter}.{extension}"));
        counter += 1;
    }
    candidate
}

fn file_stem(config: &Config) -> String {
    let label = config
        .text_zones
        .first()
        .map(|zone| zone.text.as_str())
        .filter(|text| !text.trim().is_empty())
        .unwrap_or("wallpaper");
    slug(label)
}

/// Render and write the wallpaper PNG into `dir`, returning its path.
pub fn export_png_to(dir: &Path, config: &Config) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("creating output directory {}", dir.display()))?;
    let path = unique_path(dir, &file_stem(config), "png");
    let bytes = render_png(config, config.format.longest_edge())?;
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Render and write the wallpaper PNG into [`output_dir`].
pub fn export_png(config: &Config) -> Result<PathBuf> {
    export_png_to(&output_dir(), config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("hypso-test-{nanos}-{unique}"))
    }

    fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        (width, height)
    }

    #[test]
    fn exports_desktop_at_full_resolution() {
        let dir = temp_dir();
        let path = export_png_to(&dir, &Config::default()).expect("export");
        let bytes = std::fs::read(&path).expect("read png");
        assert_eq!(png_dimensions(&bytes), (3840, 2160));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn same_config_exports_identical_image() {
        let dir = temp_dir();
        let first = export_png_to(&dir, &Config::default()).expect("export");
        let second = export_png_to(&dir, &Config::default()).expect("export");
        assert_ne!(first, second, "must not overwrite");
        assert_eq!(
            std::fs::read(&first).unwrap(),
            std::fs::read(&second).unwrap(),
            "same config ⇒ identical bytes"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn does_not_overwrite_existing_file() {
        let dir = temp_dir();
        let first = export_png_to(&dir, &Config::default()).expect("export");
        let second = export_png_to(&dir, &Config::default()).expect("export");
        assert!(first.exists() && second.exists());
        assert_ne!(first, second);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_png_wrapper_honors_output_dir_env_var() {
        let dir = temp_dir();
        // SAFETY: this is the only test that reads/writes this env var.
        unsafe { std::env::set_var("HYPSO_OUTPUT_DIR", &dir) };
        assert_eq!(output_dir(), dir);
        let path = export_png(&Config::default()).expect("export");
        unsafe { std::env::remove_var("HYPSO_OUTPUT_DIR") };
        assert!(path.starts_with(&dir) && path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

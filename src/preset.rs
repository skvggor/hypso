//! Named presets saved as TOML. A reloaded preset reproduces the same wallpaper
//! because the full configuration round-trips and every generative step is
//! seeded.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Config;
use crate::util::slug;

/// Where presets are stored. Override with `HYPSO_PRESETS_DIR`.
pub fn presets_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("HYPSO_PRESETS_DIR") {
        return PathBuf::from(custom);
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hypso")
        .join("presets")
}

fn preset_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{}.toml", slug(name)))
}

/// Save `config` as a named preset in `dir`, returning its path.
pub fn save_to(dir: &Path, name: &str, config: &Config) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("creating presets directory {}", dir.display()))?;
    let path = preset_path(dir, name);
    let text = toml::to_string_pretty(config).context("serializing preset")?;
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Load a named preset from `dir`.
pub fn load_from(dir: &Path, name: &str) -> Result<Config> {
    let path = preset_path(dir, name);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Save a named preset to [`presets_dir`].
pub fn save(name: &str, config: &Config) -> Result<PathBuf> {
    save_to(&presets_dir(), name, config)
}

/// Load a named preset from [`presets_dir`].
pub fn load(name: &str) -> Result<Config> {
    load_from(&presets_dir(), name)
}

/// Sorted names of the presets stored in `dir`.
pub fn list_from(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("toml")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    names
}

/// Sorted names of the presets in [`presets_dir`].
pub fn list() -> Vec<String> {
    list_from(&presets_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Gradient;
    use crate::render::render_png;
    use crate::text_zone::TextZone;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("hypso-preset-{nanos}-{unique}"))
    }

    fn rich_config() -> Config {
        Config {
            seed: 123,
            line_gradient: Some(Gradient {
                start: "#101010".into(),
                end: "#808080".into(),
            }),
            gradient_overlay: 0.5,
            grain: 0.25,
            text_zones: vec![TextZone {
                x: 0.1,
                y: 0.2,
                width: 0.3,
                height: 0.15,
                text: "Hello".into(),
            }],
            ..Config::default()
        }
    }

    #[test]
    fn round_trips_equal() {
        let dir = temp_dir();
        let config = rich_config();
        save_to(&dir, "my preset", &config).expect("save");
        let loaded = load_from(&dir, "my preset").expect("load");
        assert_eq!(loaded, config);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn reloaded_preset_reproduces_image() {
        let dir = temp_dir();
        let config = rich_config();
        save_to(&dir, "repro", &config).expect("save");
        let loaded = load_from(&dir, "repro").expect("load");
        assert_eq!(
            render_png(&config, 160).unwrap(),
            render_png(&loaded, 160).unwrap()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn saved_preset_file_exists() {
        let dir = temp_dir();
        let path = save_to(&dir, "saved", &Config::default()).expect("save");
        assert!(path.exists());
        assert!(path.starts_with(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_from_returns_sorted_names() {
        let dir = temp_dir();
        save_to(&dir, "beta", &Config::default()).expect("save");
        save_to(&dir, "alpha", &Config::default()).expect("save");
        assert_eq!(
            list_from(&dir),
            vec!["alpha".to_string(), "beta".to_string()]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn presets_dir_env_var_round_trips_via_wrappers() {
        let dir = temp_dir();
        // SAFETY: this is the only test that reads/writes this env var.
        unsafe { std::env::set_var("HYPSO_PRESETS_DIR", &dir) };
        assert_eq!(presets_dir(), dir);
        let config = rich_config();
        save("via env", &config).expect("save");
        let loaded = load("via env").expect("load");
        let listed = list();
        unsafe { std::env::remove_var("HYPSO_PRESETS_DIR") };
        assert_eq!(loaded, config);
        assert!(listed.contains(&"via-env".to_string()));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

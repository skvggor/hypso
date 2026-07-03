// Don't allocate a console window on Windows for the released GUI binary; keep
// it in debug builds so logs/panics stay visible during development.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use slint::{
    Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode, VecModel,
};

use hypso::config::{Config, Gradient};
use hypso::format::Format;
use hypso::text_zone::TextZone;
use hypso::{export, preset, render};

/// Preview raster bounds: never below crisp-thumbnail size, never above what a
/// live slider drag can re-render comfortably.
const PREVIEW_MIN_PIXELS: f32 = 480.0;
const PREVIEW_MAX_PIXELS: f32 = 2048.0;

/// Slint color sliders are floats; the conversion logic lives in `hypso::util`.
fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let (red, green, blue) = hypso::util::hex_to_rgb(hex);
    (red as f32, green as f32, blue as f32)
}

fn rgb_to_hex(red: f32, green: f32, blue: f32) -> String {
    let byte = |value: f32| value.round().clamp(0.0, 255.0) as u8;
    hypso::util::rgb_to_hex(byte(red), byte(green), byte(blue))
}

fn config_from_ui(ui: &AppWindow) -> Config {
    let format = Format::ALL
        .get(ui.get_format_index().max(0) as usize)
        .copied()
        .unwrap_or(Format::Desktop16x9);

    let line_gradient = if ui.get_use_gradient() {
        Some(Gradient {
            start: rgb_to_hex(
                ui.get_grad_start_red(),
                ui.get_grad_start_green(),
                ui.get_grad_start_blue(),
            ),
            end: rgb_to_hex(
                ui.get_grad_end_red(),
                ui.get_grad_end_green(),
                ui.get_grad_end_blue(),
            ),
        })
    } else {
        None
    };

    let mut text_zones = Vec::new();
    if ui.get_use_text_zone() {
        text_zones.push(TextZone {
            x: ui.get_zone_x(),
            y: ui.get_zone_y(),
            width: ui.get_zone_w(),
            height: ui.get_zone_h(),
            text: ui.get_zone_text().to_string(),
        });
    }

    Config {
        seed: ui.get_seed_text().trim().parse().unwrap_or(1),
        octaves: ui.get_octaves_value().round().max(1.0) as u32,
        frequency: ui.get_frequency_value(),
        levels: ui.get_levels_value().round().max(1.0) as u32,
        format,
        background: rgb_to_hex(ui.get_bg_red(), ui.get_bg_green(), ui.get_bg_blue()),
        line_color: rgb_to_hex(ui.get_line_red(), ui.get_line_green(), ui.get_line_blue()),
        line_gradient,
        base_stroke: ui.get_base_stroke_value(),
        index_interval: ui.get_index_interval_value().round().max(1.0) as u32,
        smoothing: ui.get_smoothing_value().round() as u32,
        gradient_overlay: ui.get_overlay_value(),
        grain: ui.get_grain_value(),
        text_zones,
        text_color: rgb_to_hex(ui.get_text_red(), ui.get_text_green(), ui.get_text_blue()),
        feather: 6.0,
    }
}

fn apply_config(ui: &AppWindow, config: &Config) {
    ui.set_seed_text(config.seed.to_string().into());
    ui.set_octaves_value(config.octaves as f32);
    ui.set_frequency_value(config.frequency);
    ui.set_levels_value(config.levels as f32);
    ui.set_format_index(
        Format::ALL
            .iter()
            .position(|f| *f == config.format)
            .unwrap_or(0) as i32,
    );
    let (bg_r, bg_g, bg_b) = hex_to_rgb(&config.background);
    ui.set_bg_red(bg_r);
    ui.set_bg_green(bg_g);
    ui.set_bg_blue(bg_b);
    let (line_r, line_g, line_b) = hex_to_rgb(&config.line_color);
    ui.set_line_red(line_r);
    ui.set_line_green(line_g);
    ui.set_line_blue(line_b);
    ui.set_use_gradient(config.line_gradient.is_some());
    if let Some(gradient) = &config.line_gradient {
        let (start_r, start_g, start_b) = hex_to_rgb(&gradient.start);
        ui.set_grad_start_red(start_r);
        ui.set_grad_start_green(start_g);
        ui.set_grad_start_blue(start_b);
        let (end_r, end_g, end_b) = hex_to_rgb(&gradient.end);
        ui.set_grad_end_red(end_r);
        ui.set_grad_end_green(end_g);
        ui.set_grad_end_blue(end_b);
    }
    ui.set_base_stroke_value(config.base_stroke);
    ui.set_index_interval_value(config.index_interval as f32);
    ui.set_smoothing_value(config.smoothing as f32);
    ui.set_overlay_value(config.gradient_overlay);
    ui.set_grain_value(config.grain);
    let (text_r, text_g, text_b) = hex_to_rgb(&config.text_color);
    ui.set_text_red(text_r);
    ui.set_text_green(text_g);
    ui.set_text_blue(text_b);
    if let Some(zone) = config.text_zones.first() {
        ui.set_use_text_zone(true);
        ui.set_zone_text(zone.text.clone().into());
        ui.set_zone_x(zone.x);
        ui.set_zone_y(zone.y);
        ui.set_zone_w(zone.width);
        ui.set_zone_h(zone.height);
    }
}

/// The preview is rendered at the exact physical size it is displayed at
/// (`image-fit: contain` inside the preview area), so pixels map 1:1.
fn preview_pixels(ui: &AppWindow, config: &Config) -> u32 {
    let scale_factor = ui.window().scale_factor();
    let area_width = ui.get_preview_area_width() * scale_factor;
    let area_height = ui.get_preview_area_height() * scale_factor;
    let (format_width, format_height) = config.format.dimensions();
    let fit = (area_width / format_width as f32).min(area_height / format_height as f32);
    (config.format.longest_edge() as f32 * fit)
        .clamp(PREVIEW_MIN_PIXELS, PREVIEW_MAX_PIXELS)
        .round() as u32
}

fn refresh_preview(ui: &AppWindow) {
    let config = config_from_ui(ui);
    match render::render_rgba(&config, preview_pixels(ui, &config)) {
        Ok((width, height, rgba)) => {
            let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
            buffer.make_mut_bytes().copy_from_slice(&rgba);
            ui.set_preview(Image::from_rgba8(buffer));
        }
        Err(error) => ui.set_status(SharedString::from(format!("Preview error: {error}"))),
    }
}

fn string_model(items: Vec<String>) -> ModelRc<SharedString> {
    Rc::new(VecModel::from(
        items
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    ))
    .into()
}

fn reload_presets(model: &VecModel<SharedString>) {
    model.set_vec(
        preset::list()
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<_>>(),
    );
}

/// Show the exported file in the platform file manager. Windows Explorer can
/// select the file itself; elsewhere the containing folder is opened.
fn reveal_in_file_manager(path: &std::path::Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let directory = path.parent().filter(|p| !p.as_os_str().is_empty());
        open::that_detached(directory.unwrap_or(path))?;
        Ok(())
    }
}

fn default_to_software_renderer() {
    if std::env::var_os("SLINT_BACKEND").is_none() {
        // SAFETY: set before any backend/window initialization, still single-threaded.
        unsafe { std::env::set_var("SLINT_BACKEND", "winit-software") };
    }
}

fn register_color_codec(ui: &AppWindow) {
    let byte = |value: f32| value.round().clamp(0.0, 255.0) as u8;
    let codec = ui.global::<ColorCodec>();
    codec.on_parse(|text| match hypso::util::parse_color_text(&text) {
        Some((red, green, blue)) => RgbValue {
            red: red as f32,
            green: green as f32,
            blue: blue as f32,
            valid: true,
        },
        None => RgbValue {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            valid: false,
        },
    });
    codec.on_format_hex(move |red, green, blue| {
        hypso::util::rgb_to_hex(byte(red), byte(green), byte(blue)).into()
    });
    codec.on_format_rgb(move |red, green, blue| {
        hypso::util::rgb_to_text(byte(red), byte(green), byte(blue)).into()
    });
}

fn main() -> Result<()> {
    default_to_software_renderer();

    let ui = AppWindow::new()?;
    register_color_codec(&ui);

    ui.set_formats(string_model(
        Format::ALL.iter().map(|f| f.label().to_string()).collect(),
    ));

    let presets = Rc::new(VecModel::<SharedString>::default());
    ui.set_presets(presets.clone().into());
    reload_presets(&presets);

    apply_config(&ui, &Config::default());
    // Sensible starting text-zone box (normalized), hidden until enabled.
    ui.set_zone_x(0.08);
    ui.set_zone_y(0.10);
    ui.set_zone_w(0.40);
    ui.set_zone_h(0.18);
    // Sensible gradient colors, shown once the gradient is enabled.
    ui.set_grad_start_red(27.0);
    ui.set_grad_start_green(27.0);
    ui.set_grad_start_blue(27.0);
    ui.set_grad_end_red(111.0);
    ui.set_grad_end_green(182.0);
    ui.set_grad_end_blue(200.0);

    // Debounce rapid edits into one render ~50 ms after the user stops.
    let preview_timer = Rc::new(Timer::default());
    ui.on_changed({
        let handle = ui.as_weak();
        let timer = preview_timer.clone();
        move || {
            let handle = handle.clone();
            timer.start(
                TimerMode::SingleShot,
                Duration::from_millis(50),
                move || {
                    if let Some(ui) = handle.upgrade() {
                        refresh_preview(&ui);
                    }
                },
            );
        }
    });

    ui.on_randomize({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                ui.set_seed_text(fastrand::u64(..).to_string().into());
                refresh_preview(&ui);
            }
        }
    });

    ui.on_export_png({
        let handle = ui.as_weak();
        move || {
            let Some(ui) = handle.upgrade() else { return };
            let config = config_from_ui(&ui);
            ui.set_exporting(true);
            ui.set_export_overlay(true);
            ui.set_export_done(false);
            ui.set_export_path(SharedString::new());
            ui.set_export_error(SharedString::new());
            let weak = handle.clone();
            std::thread::spawn(move || {
                let result = export::export_png(&config);
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_exporting(false);
                        ui.set_export_done(true);
                        match result {
                            Ok(path) => ui.set_export_path(path.display().to_string().into()),
                            Err(error) => ui.set_export_error(error.to_string().into()),
                        }
                    }
                })
                .ok();
            });
        }
    });

    ui.on_open_export_path({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                let path = ui.get_export_path().to_string();
                if let Err(error) = reveal_in_file_manager(std::path::Path::new(&path)) {
                    ui.set_status(format!("Could not open file manager: {error}").into());
                }
            }
        }
    });

    ui.on_dismiss_export({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                ui.set_export_overlay(false);
            }
        }
    });

    ui.on_save_preset({
        let handle = ui.as_weak();
        let presets = presets.clone();
        move || {
            if let Some(ui) = handle.upgrade() {
                let typed = ui.get_preset_name().to_string();
                let name = if typed.trim().is_empty() {
                    "wallpaper".to_string()
                } else {
                    typed
                };
                let status = match preset::save(&name, &config_from_ui(&ui)) {
                    Ok(path) => {
                        reload_presets(&presets);
                        format!("Saved preset → {}", path.display())
                    }
                    Err(error) => format!("Save preset failed: {error}"),
                };
                ui.set_status(status.into());
            }
        }
    });

    ui.on_load_preset({
        let handle = ui.as_weak();
        move || {
            if let Some(ui) = handle.upgrade() {
                let names = preset::list();
                let index = ui.get_preset_index().max(0) as usize;
                let Some(name) = names.get(index) else {
                    ui.set_status("Select a saved preset first".into());
                    return;
                };
                match preset::load(name) {
                    Ok(config) => {
                        apply_config(&ui, &config);
                        refresh_preview(&ui);
                        ui.set_status(format!("Loaded preset \"{name}\"").into());
                    }
                    Err(error) => ui.set_status(format!("Load preset failed: {error}").into()),
                }
            }
        }
    });

    refresh_preview(&ui);
    ui.run()?;
    Ok(())
}

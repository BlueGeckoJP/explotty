use std::{
    path::{Path, PathBuf},
    process::Command,
};

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use gio::glib::object::Cast;
use gtk::traits::IconThemeExt;

use crate::CONFIG;

// Unix-like systems only
pub fn get_current_dir_from_pty(pid: u32) -> Option<PathBuf> {
    #[cfg(unix)]
    {
        let cwd_path = format!("/proc/{pid}/cwd");
        std::fs::read_link(cwd_path).ok()
    }

    #[cfg(not(unix))]
    {
        warn!("get_current_dir_from_pty is only implemented for Unix-like systems");
        None
    }
}

pub fn load_system_font(ctx: &Context) -> anyhow::Result<()> {
    let mut fonts = FontDefinitions::default();

    let (sans_serif_family, monospace_family) = if let Some(config) = CONFIG.get() {
        (
            FamilyName::Title(config.ui_font_family.clone().unwrap_or_default()),
            FamilyName::Title(config.terminal_font_family.clone().unwrap_or_default()),
        )
    } else {
        (FamilyName::SansSerif, FamilyName::Monospace)
    };

    let terminal_fallback_fonts = if let Some(config) = CONFIG.get() {
        config
            .terminal_fallback_font_families
            .clone()
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let sans_serif_handle =
        SystemSource::new().select_best_match(&[sans_serif_family], &Properties::new())?;
    let sans_serif_buf: Vec<u8> = match sans_serif_handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => std::fs::read(path)?,
    };

    let monospace_handle =
        SystemSource::new().select_best_match(&[monospace_family], &Properties::new())?;
    let monospace_buf: Vec<u8> = match monospace_handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => std::fs::read(path)?,
    };

    let terminal_fallback_buffers = terminal_fallback_fonts
        .iter()
        .map(|family| {
            let handle = SystemSource::new()
                .select_best_match(&[FamilyName::Title(family.clone())], &Properties::new())?;
            match handle {
                Handle::Memory { bytes, .. } => Ok(bytes.to_vec()),
                Handle::Path { path, .. } => Ok(std::fs::read(path)?),
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    const SANS_SERIF_FONT_ID: &str = "System Sans Serif";
    const MONOSPACE_FONT_ID: &str = "System Monospace";

    fonts.font_data.insert(
        SANS_SERIF_FONT_ID.to_string(),
        FontData::from_owned(sans_serif_buf).into(),
    );
    fonts.font_data.insert(
        MONOSPACE_FONT_ID.to_string(),
        FontData::from_owned(monospace_buf).into(),
    );
    fonts.font_data.extend(
        terminal_fallback_buffers
            .into_iter()
            .enumerate()
            .map(|(i, buf)| {
                (
                    terminal_fallback_fonts[i].clone(),
                    FontData::from_owned(buf).into(),
                )
            }),
    );

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.insert(0, SANS_SERIF_FONT_ID.to_string());
        info!("Proportional font family: {vec:?}");
    }
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        let mut fonts = vec![MONOSPACE_FONT_ID.to_string()];
        fonts.extend(terminal_fallback_fonts.iter().cloned());
        vec.splice(0..0, fonts);
        info!("Monospace font family: {vec:?}");
    }

    ctx.set_fonts(fonts);

    Ok(())
}

pub fn to_human_readable_size(size: u64) -> String {
    if size < 1024 {
        format!("{size} B")
    } else if size < 1024 * 1024 {
        format!("{:.2} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!(
            "{:.2} TB",
            size as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
        )
    }
}

pub fn get_mime_type_from_path(path: &Path) -> String {
    match path.is_dir() {
        true => "inode/directory".to_string(),
        false => {
            let (content_type, _) = gio::content_type_guess(Some(path), None);
            content_type.to_string()
        }
    }
}

fn find_icon(mime_type: &str, size: i32) -> Option<String> {
    let icon = gio::content_type_get_icon(mime_type);

    if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>() {
        let icon_names = themed_icon.names();
        let icon_theme = gtk::IconTheme::default();

        for name in icon_names {
            if let Some(icon_info) =
                icon_theme
                    .clone()?
                    .lookup_icon(&name, size, gtk::IconLookupFlags::empty())
                && let Some(filename) = icon_info.filename()
            {
                return Some(filename.to_string_lossy().to_string());
            }
        }
    }

    None
}

pub fn get_formatted_icon_path(mime_type: &str, size: i32) -> String {
    format!("file://{}", find_icon(mime_type, size).unwrap_or_default())
}

pub fn get_desc_from_mime_type(mime_type: &str) -> String {
    let desc = gio::content_type_get_description(mime_type);
    desc.to_string()
}

pub fn copy_file_uri_to_clipboard(path: &str) {
    let uri = format!("file://{path}").replace("\'", "'\\''");

    let commandline = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        format!("echo {uri} | wl-copy --type text/uri-list")
    } else {
        format!("echo {uri} | xclip -selection clipboard -t text/uri-list")
    };

    match Command::new("sh").arg("-c").arg(&commandline).status() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            error!("Clipboard copy failed. Command exited with status: {status}");
        }
        Err(e) => {
            error!("Failed to spawn clipboard command: {e}")
        }
    }
}

use std::path::Path;

use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};
use font_kit::{
    family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
};
use gio::glib::object::Cast;
use gtk::traits::IconThemeExt;

// Unix-like systems only
pub fn get_current_dir_from_pty(pid: u32) -> Option<String> {
    #[cfg(unix)]
    {
        let cwd_path = format!("/proc/{pid}/cwd");
        match std::fs::read_link(cwd_path) {
            Ok(path) => Some(path.to_string_lossy().into_owned()),
            Err(_) => None,
        }
    }

    #[cfg(not(unix))]
    {
        warn!("get_current_dir_from_pty is only implemented for Unix-like systems");
        None
    }
}

pub fn load_system_font(ctx: &Context) -> anyhow::Result<()> {
    let handle =
        SystemSource::new().select_best_match(&[FamilyName::Monospace], &Properties::new())?;

    let buf: Vec<u8> = match handle {
        Handle::Memory { bytes, .. } => bytes.to_vec(),
        Handle::Path { path, .. } => std::fs::read(path)?,
    };

    let mut fonts = FontDefinitions::default();

    const FONT_ID: &str = "System Sans Serif";

    fonts
        .font_data
        .insert(FONT_ID.to_string(), FontData::from_owned(buf).into());

    if let Some(vec) = fonts.families.get_mut(&FontFamily::Proportional) {
        vec.insert(0, FONT_ID.to_string());
    }
    if let Some(vec) = fonts.families.get_mut(&FontFamily::Monospace) {
        vec.insert(0, FONT_ID.to_string());
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

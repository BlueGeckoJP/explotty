use gio::prelude::*;
use gtk::prelude::*;
use std::path::Path;

pub fn find_icon(filename: &str, size: i32) -> Option<String> {
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK");
        return None;
    }

    let file_path = Path::new(filename);
    let content_type = match file_path.is_dir() {
        true => "inode/directory".to_string(),
        false => {
            let (content_type, _) = gio::content_type_guess(Some(file_path), None);
            content_type.to_string()
        }
    };

    let icon = gio::content_type_get_icon(&content_type);

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

use eframe::egui::Color32;

pub fn process_256_color_palette(color_index: u8) -> Color32 {
    if color_index < 16 {
        // 16 basic colors
        match color_index {
            0 => Color32::BLACK,
            1 => Color32::RED,
            2 => Color32::GREEN,
            3 => Color32::YELLOW,
            4 => Color32::BLUE,
            5 => Color32::MAGENTA,
            6 => Color32::CYAN,
            7 => Color32::WHITE,
            8 => to_bright(Color32::BLACK),
            9 => to_bright(Color32::RED),
            10 => to_bright(Color32::GREEN),
            11 => to_bright(Color32::YELLOW),
            12 => to_bright(Color32::BLUE),
            13 => to_bright(Color32::MAGENTA),
            14 => to_bright(Color32::CYAN),
            15 => to_bright(Color32::WHITE),
            _ => unreachable!(),
        }
    } else if (16..232).contains(&color_index) {
        // 6x6x6 rgb color cube
        let r_6 = (color_index - 16) / 36;
        let g_6 = ((color_index - 16) % 36) / 6;
        let b_6 = (color_index - 16) % 6;

        let rgb: (u8, u8, u8) = [r_6, g_6, b_6]
            .map(|x| match x {
                0 => 0,
                1 => 95,
                2 => 135,
                3 => 175,
                4 => 215,
                5 => 255,
                _ => unreachable!(),
            })
            .into();

        Color32::from_rgb(rgb.0, rgb.1, rgb.2)
    } else {
        // 232..=255
        // Grayscale colors
        let gray_value = (color_index - 232) * 10 + 8; // 8, 18, ..., 238
        Color32::from_gray(gray_value)
    }
}

pub fn to_bright(color: Color32) -> Color32 {
    let rgb = color.to_array();
    Color32::from_rgb(
        (rgb[0] as f32 * 1.2).min(255.0) as u8,
        (rgb[1] as f32 * 1.2).min(255.0) as u8,
        (rgb[2] as f32 * 1.2).min(255.0) as u8,
    )
}

use eframe::egui;

pub fn custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "unifont".into(),
        egui::FontData::from_static(include_bytes!("../assets/unifont-15.0.06.ttf")),
    );
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "unifont".into());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "unifont".to_owned());
    ctx.set_fonts(fonts);
}

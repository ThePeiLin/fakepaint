use eframe::egui;

pub struct TileSet {
    pub image_data: image::DynamicImage,
    pub tex: egui::TextureHandle,
    uv: Vec<egui::Rect>,
    pub columns: usize,
    pub rows: usize,
    pub tile_size: egui::Vec2,
}

fn load_image_from_path(
    path: &std::path::Path,
) -> Result<(egui::ColorImage, image::DynamicImage), image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let img_buf = image.to_rgba8();
    let pixels = img_buf.as_flat_samples();
    Ok((
        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
        image,
    ))
}

pub fn load_texture(ctx: &egui::Context) -> (Option<egui::TextureHandle>, image::DynamicImage) {
    let mut ascii: Option<egui::TextureHandle> = None;
    let tex_opt = egui::TextureOptions {
        magnification: egui::TextureFilter::Nearest,
        minification: egui::TextureFilter::Nearest,
    };
    let (color_image, image) =
        load_image_from_path(std::path::Path::new("assets/16x16_sm_ascii.png")).unwrap();
    ascii.get_or_insert_with(|| ctx.load_texture("16x16_sm_ascii", color_image, tex_opt));
    (ascii, image)
}

impl TileSet {
    pub fn new(
        image_data: image::DynamicImage,
        tex: egui::TextureHandle,
        columns: usize,
        rows: usize,
        tile_size: egui::Vec2,
    ) -> Self {
        let ux = 1.0 / (columns as f32);
        let uy = 1.0 / (rows as f32);
        let mut uv = Vec::with_capacity(columns * rows);
        for i in 0..rows {
            for j in 0..columns {
                let x = ux * j as f32;
                let y = uy * i as f32;
                uv.push(egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::Vec2::new(ux, uy),
                ));
            }
        }
        Self {
            image_data,
            tex,
            uv,
            columns,
            rows,
            tile_size,
        }
    }

    #[allow(unused)]
    pub fn uv(&self, idx: usize) -> egui::Rect {
        self.uv[idx]
    }

    #[allow(unused)]
    pub fn to_image(&self, idx: usize, size: egui::Vec2) -> egui::Image {
        egui::Image::new(self.tex.id(), size).uv(self.uv(idx))
    }

    #[allow(unused)]
    pub fn to_image_xy(&self, idx: usize, sx: f32, sy: f32) -> egui::Image {
        egui::Image::new(self.tex.id(), egui::vec2(sx, sy)).uv(self.uv(idx))
    }

    #[allow(unused)]
    pub fn paint_in_rect(
        &self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        idx: usize,
        tint: egui::Color32,
        bg: Option<egui::Color32>,
    ) {
        if ui.is_rect_visible(rect) {
            if let Some(bg) = bg {
                ui.painter().rect_filled(rect, egui::Rounding::none(), bg);
            }
            ui.painter().image(self.tex.id(), rect, self.uv(idx), tint);
        }
    }
}

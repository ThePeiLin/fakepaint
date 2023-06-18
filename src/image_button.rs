use eframe::egui;
use egui::TextureId;

pub struct ImageButton<'a> {
    tex: Option<TextureId>,
    uv: egui::Rect,
    size: egui::Vec2,
    tint: egui::Color32,
    selected_tint: egui::Color32,
    bg_fill: Option<egui::Color32>,
    selected_bg_fill: Option<egui::Color32>,
    sense: egui::Sense,
    frame: bool,
    selected: bool,
    need_rounding: bool,
    on: Option<&'a mut bool>,
}

impl<'a> ImageButton<'a> {
    pub fn new(tex: Option<TextureId>, size: egui::Vec2) -> Self {
        Self {
            tex,
            uv: egui::Rect::from_two_pos(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            size,
            tint: egui::Color32::WHITE,
            selected_tint: egui::Color32::WHITE,
            sense: egui::Sense::click(),
            frame: true,
            selected: false,
            bg_fill: None,
            selected_bg_fill: None,
            need_rounding: true,
            on: None,
        }
    }

    #[allow(unused)]
    pub fn on(mut self, on: &'a mut bool) -> Self {
        self.on = Some(on);
        self
    }

    #[allow(unused)]
    pub fn uv(mut self, uv: impl Into<egui::Rect>) -> Self {
        self.uv = uv.into();
        self
    }

    #[allow(unused)]
    pub fn tex(mut self, tex: TextureId) -> Self {
        self.tex = Some(tex);
        self
    }

    #[allow(unused)]
    pub fn tint(mut self, tint: impl Into<egui::Color32>) -> Self {
        self.tint = tint.into();
        self
    }

    #[allow(unused)]
    pub fn rounding(mut self, rounding: bool) -> Self {
        self.need_rounding = rounding;
        self
    }

    #[allow(unused)]
    pub fn selected_tint(mut self, tint: impl Into<egui::Color32>) -> Self {
        self.selected_tint = tint.into();
        self
    }

    #[allow(unused)]
    pub fn bg_fill(mut self, bg_fill: impl Into<egui::Color32>) -> Self {
        self.bg_fill = Some(bg_fill.into());
        self
    }

    #[allow(unused)]
    pub fn selected_bg_fill(mut self, bg_fill: impl Into<egui::Color32>) -> Self {
        self.selected_bg_fill = Some(bg_fill.into());
        self
    }

    #[allow(unused)]
    pub fn size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }

    #[allow(unused)]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[allow(unused)]
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    #[allow(unused)]
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl egui::Widget for ImageButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            tex,
            uv,
            size,
            tint,
            sense,
            frame,
            selected,
            selected_tint,
            bg_fill,
            selected_bg_fill,
            need_rounding,
            on,
        } = self;
        let padding = if frame {
            egui::Vec2::splat(ui.spacing().button_padding.x)
        } else {
            egui::Vec2::ZERO
        };
        let padding_size = size + 2.0 * padding;
        let (rect, mut response) = ui.allocate_exact_size(padding_size, sense);

        let mut on1 = false;
        if let Some(on) = on {
            if response.clicked() {
                on1 = !*on;
                *on = on1;
                response.mark_changed();
            }
        }
        let on1 = on1;

        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::ImageButton, self.selected, "")
        });
        if ui.is_rect_visible(rect) {
            let (expansion, rounding, fill, stroke) = if selected {
                let selection = ui.visuals().selection;
                let bg_fill = if on1 {
                    ui.visuals().selection.bg_fill
                } else if let Some(bgc) = selected_bg_fill {
                    bgc
                } else {
                    selection.bg_fill
                };
                (
                    egui::Vec2::ZERO,
                    egui::Rounding::none(),
                    bg_fill,
                    selection.stroke,
                )
            } else if frame {
                let visuals = ui.style().interact(&response);
                let expansion = egui::Vec2::splat(visuals.expansion);
                let bg_fill = if on1 {
                    ui.visuals().selection.bg_fill
                } else if let Some(bg) = bg_fill {
                    bg
                } else {
                    if response.has_focus() || response.hovered() {
                        visuals.bg_fill
                    } else {
                        egui::Color32::TRANSPARENT
                    }
                };
                (expansion, visuals.rounding, bg_fill, visuals.bg_stroke)
            } else {
                let visuals = ui.style().interact(&response);
                let bg_fill = if on1 {
                    ui.visuals().selection.bg_fill
                } else if let Some(bg) = bg_fill {
                    bg
                } else if response.has_focus() || response.hovered() {
                    visuals.bg_fill
                } else {
                    egui::Color32::TRANSPARENT
                };

                (
                    Default::default(),
                    visuals.rounding,
                    bg_fill,
                    visuals.bg_stroke,
                )
            };

            let rounding = if need_rounding {
                rounding
            } else {
                egui::Rounding::none()
            };
            ui.painter()
                .rect_filled(rect.expand2(expansion), rounding, fill);
            let image_rect = ui
                .layout()
                .align_size_within_rect(size, rect.shrink2(padding));

            if let Some(tex) = tex {
                ui.painter().image(
                    tex,
                    image_rect,
                    uv,
                    if selected { selected_tint } else { tint },
                );
            }
            ui.painter()
                .rect_stroke(rect.expand2(expansion), rounding, stroke);
        }
        response
    }
}

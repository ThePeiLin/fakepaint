#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod setup;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    let opts = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "fakepaint",
        opts,
        Box::new(|cc| Box::new(FakePaint::new(cc))),
    )
}

struct FakePaint {
    greeting: String,
}

impl FakePaint {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup::custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            greeting: "hello, world!".into(),
        }
    }
}

impl eframe::App for FakePaint {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.greeting.clone());
            ui.label(self.greeting.clone());
        });
    }
}


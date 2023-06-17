#![cfg_attr(not(debug_assertions),windows_subsystem="windows")]

use eframe::egui;

fn main() ->Result<(),eframe::Error> {
    let opts=eframe::NativeOptions{
        initial_window_size:Some(egui::vec2(320.0,240.0)),
        ..Default::default()
    };
    eframe::run_native("fakepaint",
                       opts,
                       Box::new(|_cc|Box::<FakePaint>::default()),
                       )
}

struct FakePaint{
    greeting:String,
}

impl Default for FakePaint{
    fn default()->Self{
        Self{
            greeting:"hello, world!".into(),
        }
    }
}

impl eframe::App for FakePaint{
    fn update(&mut self,
              ctx:&egui::Context,
              _frame:&mut eframe::Frame,){
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.heading(self.greeting.clone());
            ui.label(self.greeting.clone());
        });
    }
}

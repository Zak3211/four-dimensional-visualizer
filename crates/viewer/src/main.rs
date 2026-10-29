use eframe::egui;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "4D visualizer",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(Smoke))),
    )
}

struct Smoke;
impl eframe::App for Smoke {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| ui.heading("smoke"));
    }
}

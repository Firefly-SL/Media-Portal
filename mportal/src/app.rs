use eframe::egui::{self};

use crate::fonts;

pub struct App {
}

impl App {
    pub fn new(ctx: &eframe::egui::Context) -> Self {
        fonts::configure_fonts(ctx);
        
        Self {
            
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
    }
}
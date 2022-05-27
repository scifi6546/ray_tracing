use super::{LogMessage, Message, RayTracerInfo};
use egui_miniquad as egui_mq;
use miniquad::Context;
use std::sync::mpsc::{Receiver, Sender};
pub struct GuiCtx {
    egui_mq: egui_mq::EguiMq,
    log_reciever: Receiver<LogMessage>,
    scenarios: Vec<String>,
    message_chanel: Sender<Message>,
    log_messages: Vec<LogMessage>,
}
impl GuiCtx {
    pub fn new(
        ctx: &mut miniquad::Context,
        info: &RayTracerInfo,
        log_reciever: Receiver<LogMessage>,
        message_chanel: Sender<Message>,
    ) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            log_reciever,
            scenarios: info.scenarios.clone(),
            log_messages: vec![],
            message_chanel,
        }
    }
    pub fn update(&mut self, ctx: &mut miniquad::Context) {
        for msg in self.log_reciever.try_iter() {
            self.log_messages.push(msg);
        }
        self.egui_mq.run(ctx, |egui_ctx| {
            egui::Window::new("Hello world").show(egui_ctx, |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.heading("Scenario: ");
                for scenario in self.scenarios.iter() {
                    if ui.button(scenario).clicked() {
                        self.message_chanel
                            .send(Message::LoadScenario(scenario.clone()));
                        println!("clicked: {}", scenario);
                    }
                }
            });
            egui::Window::new("Log")
                .vscroll(false)
                .hscroll(false)
                .default_height(300.0)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::new([true, true])
                        .stick_to_bottom()
                        .show(ui, |ui| {
                            egui::Grid::new("Log")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    for msg in self.log_messages.iter() {
                                        let (log_level, text) = match msg {
                                            LogMessage::Debug(s) => (
                                                egui::RichText::new("Debug")
                                                    .color(egui::Rgba::from_rgb(0.3, 1.0, 0.3)),
                                                egui::RichText::new(s)
                                                    .color(egui::Rgba::from_gray(0.8)),
                                            ),
                                            LogMessage::Info(s) => (
                                                egui::RichText::new("Info")
                                                    .color(egui::Rgba::from_rgb(0.3, 0.3, 1.0)),
                                                egui::RichText::new(s)
                                                    .color(egui::Rgba::from_gray(0.8)),
                                            ),
                                            LogMessage::Warn(s) => (
                                                egui::RichText::new("Warn")
                                                    .color(egui::Rgba::from_rgb(0.3, 0.1, 0.3)),
                                                egui::RichText::new(s)
                                                    .color(egui::Rgba::from_gray(0.8)),
                                            ),
                                            LogMessage::Error(s) => (
                                                egui::RichText::new("Warn")
                                                    .color(egui::Rgba::from_rgb(1.0, 0.3, 0.3)),
                                                egui::RichText::new(s)
                                                    .color(egui::Rgba::from_gray(0.8)),
                                            ),
                                            _ => todo!(),
                                        };
                                        ui.add(egui::Label::new(log_level));
                                        ui.add(egui::Label::new(text));
                                        ui.end_row();
                                    }
                                });
                        });
                });
            ()
        });
    }
    pub fn draw(&mut self, ctx: &mut miniquad::Context) {
        self.egui_mq.draw(ctx);
    }
    pub fn mouse_motion_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(ctx, x, y);
    }
    pub fn mouse_wheel_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_wheel_event(ctx, x, y);
    }
    pub fn mouse_button_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    pub fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    pub fn char_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        character: char,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    pub fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }
    pub fn key_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
    ) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}

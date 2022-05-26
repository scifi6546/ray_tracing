use super::{Message, RayTracerInfo};
use egui_miniquad as egui_mq;
use miniquad::Context;
use std::sync::mpsc::Sender;
pub struct GuiCtx {
    egui_mq: egui_mq::EguiMq,
    scenarios: Vec<String>,
    message_chanel: Sender<Message>,
}
impl GuiCtx {
    pub fn new(
        ctx: &mut miniquad::Context,
        info: &RayTracerInfo,
        message_chanel: Sender<Message>,
    ) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            scenarios: info.scenarios.clone(),
            message_chanel,
        }
    }
    pub fn update(&mut self, ctx: &mut miniquad::Context) {
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

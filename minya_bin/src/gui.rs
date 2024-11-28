mod gui_state;

use crate::messages::{GuiPushMessage, GuiSendMessage};
use egui_miniquad as egui_mq;

use gui_state::GuiState;
use lib_minya::ray_tracer::ray_tracer_info::RayTracerInfo;

use std::sync::mpsc::{Receiver, Sender};
pub struct GuiCtx {
    egui_mq: egui_mq::EguiMq,
    receiver_messages: Receiver<GuiSendMessage>,
    state: GuiState,
}
impl GuiCtx {
    pub fn new<'a>(
        ctx: &'a mut miniquad::Context,
        info: &'a RayTracerInfo,
        push_message_chanel: Sender<GuiPushMessage>,
        receiver_messages: Receiver<GuiSendMessage>,
    ) -> Self {
        let mut scenarios = info.scenarios.clone();
        scenarios.sort();

        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            state: GuiState::new(info, push_message_chanel),
            receiver_messages,
        }
    }
    pub fn update(&mut self, ctx: &mut miniquad::Context) {
        for message in self.receiver_messages.try_iter() {
            match message {
                GuiSendMessage::UpdateRayTracerInfo(info) => self.state.set_ray_tracer_info(info),
            }
        }
        self.egui_mq.run(ctx, |_mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("top_menu").show(egui_ctx, |ui| {
                self.state.top_menu(ui);
            });
            egui::SidePanel::left("entity viewer").show(egui_ctx, |ui| {
                self.state.entity_side_bar(ui);
            });

            egui::Window::new("Log")
                .vscroll(false)
                .hscroll(false)
                .default_height(300.0)
                .show(egui_ctx, |ui| {
                    self.state.log_window(ui);
                });
            egui::TopBottomPanel::bottom("play pause").show(egui_ctx, |ui| {
                let _pressed = ui.button("PAUSE").changed();
            });
        });
    }

    pub fn draw<'a>(&mut self, ctx: &'a mut miniquad::Context) {
        self.egui_mq.draw(ctx);
    }
    pub fn mouse_motion_event(&mut self, _ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }
    pub fn mouse_wheel_event(&mut self, _ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_wheel_event(x, y);
    }
    pub fn mouse_button_down_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    pub fn mouse_button_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
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
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(keycode, keymods);
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

use super::{LogMessage, Message, RayTracerInfo};
use egui_miniquad as egui_mq;
use lib_minya::ray_tracer::CurrentShader;
use log::{error, info};
use std::sync::mpsc::{Receiver, Sender};
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
        self.egui_mq.run(ctx, |mq_ctx, egui_ctx| {
            egui::Window::new("Scenarios").show(egui_ctx, |ui| {
                ui.heading("Choose Scenario: ");
                for scenario in self.scenarios.iter() {
                    if ui.button(scenario).clicked() {
                        if let Some(err) = self
                            .message_chanel
                            .send(Message::LoadScenario(scenario.clone()))
                            .err()
                        {
                            error!("failed to load scenario: {:?}", err);
                        }
                        println!("clicked: {}", scenario);
                    }
                }
            });
            egui::Window::new("Save File").show(egui_ctx, |ui| {
                ui.separator();
                ui.label(egui::RichText::new("Save").heading());
                if ui.button("Save Render").clicked() {
                    info!("saving file");
                    if let Some(save_path) = rfd::FileDialog::new().save_file() {
                        if let Some(err) = self
                            .message_chanel
                            .send(Message::SaveFile(save_path.clone()))
                            .err()
                        {
                            error!(
                                "failed to save file because rendering thread crashed, error: {:?}",
                                err
                            )
                        }
                        info!("saving file to path: {:?}", save_path);
                    } else {
                        info!("file save canceled");
                    }
                }
                ui.separator();
                ui.label(egui::RichText::new("Set Shader").heading());
                let shaders = CurrentShader::names();
                for s in shaders {
                    if ui.button(&s).clicked() {
                        self.message_chanel
                            .send(Message::SetShader(CurrentShader::from_str(&s)))
                            .expect("failed to send");
                    }
                }
            });

            egui::Window::new("Log")
                .vscroll(false)
                .hscroll(false)
                .default_height(300.0)
                .show(egui_ctx, |ui| {
                    egui::ScrollArea::new([true, true])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            egui::Grid::new("Log")
                                .num_columns(2)
                                .striped(true)
                                .show(ui, |ui| {
                                    let log_messages =
                                        lib_minya::ray_tracer::logger::Logger::get_log_messages();

                                    for msg in log_messages.iter() {
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
                                                egui::RichText::new("Error")
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
        self.egui_mq.mouse_motion_event(x, y);
    }
    pub fn mouse_wheel_event(&mut self, ctx: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_wheel_event(x, y);
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

use super::{GuiPushMessage, LogMessage};
use egui_miniquad as egui_mq;

use cgmath::{Point3, Vector3};
use lib_minya::ray_tracer::{
    ray_tracer_info::{Entity, EntityField, RayTracerInfo, ScenarioInfo},
    CurrentShader,
};
use log::{error, info};
use std::sync::mpsc::Sender;

pub struct GuiCtx {
    egui_mq: egui_mq::EguiMq,

    state: GuiState,
}
impl GuiCtx {
    pub fn new<'a>(
        ctx: &'a mut miniquad::Context,
        info: &'a RayTracerInfo,
        message_chanel: Sender<GuiPushMessage>,
    ) -> Self {
        let mut scenarios = info.scenarios.clone();
        scenarios.sort();

        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
            state: GuiState::new(info, message_chanel),
        }
    }
    pub fn update(&mut self, ctx: &mut miniquad::Context) {
        self.egui_mq.run(ctx, |_mq_ctx, egui_ctx| {
            egui::TopBottomPanel::top("top_menu").show(egui_ctx, |ui| {
                self.state.top_menu(ui);
            });
            egui::SidePanel::left("entity viewer").show(egui_ctx, |ui| {
                self.state.entity_menu(ui);
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
struct GuiState {
    info: RayTracerInfo,
    message_chanel: Sender<GuiPushMessage>,
}
impl GuiState {
    fn new(info: &RayTracerInfo, message_chanel: Sender<GuiPushMessage>) -> Self {
        let mut info = info.clone();
        info.scenarios.sort();
        Self {
            info,
            message_chanel,
        }
    }
    fn top_menu(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Save Render").clicked() {
                    info!("saving file");
                    if let Some(save_path) = rfd::FileDialog::new().save_file() {
                        if let Some(err) = self
                            .message_chanel
                            .send(GuiPushMessage::SaveFile(save_path.clone()))
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
            });
            ui.menu_button("Scenarios", |ui| {
                for scenario in self.info.scenarios.iter() {
                    if ui.button(&scenario.name).clicked() {
                        if let Some(err) = self
                            .message_chanel
                            .send(GuiPushMessage::LoadScenario(scenario.name.clone()))
                            .err()
                        {
                            error!("failed to load scenario: {:?}", err);
                        }
                        info!("clicked: {:?}", scenario);
                    }
                }
            });
            ui.menu_button("Shader", |ui| {
                let shaders = CurrentShader::names();
                for s in shaders {
                    if ui.button(&s).clicked() {
                        self.message_chanel
                            .send(GuiPushMessage::SetShader(CurrentShader::from_str(&s)))
                            .expect("failed to send");
                    }
                }
            });
        });
    }
    fn entity_menu(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Main Camera")
            .default_open(true)
            .show(ui, |ui| {
                let fields_map = self.info.loaded_entities.main_camera.fields();
                let mut fields = fields_map.iter().collect::<Vec<_>>();
                fields.sort_by(|(key_a, _value_a), (key_b, _value_b)| key_a.cmp(key_b));
                for (field_name, field_info) in fields {
                    ui.label(field_name);
                    match field_info {
                        EntityField::Point3(point) => {
                            let mut x = point.x;
                            ui.label("x");

                            ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                            let mut y = point.y;
                            ui.label("y");
                            ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();
                            let mut z = point.z;
                            ui.label("z");
                            ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed();
                            if x != point.x || y != point.y || z != point.z {
                                self.message_chanel
                                    .send(GuiPushMessage::SetCameraData((
                                        field_name.clone(),
                                        EntityField::Point3(Point3::new(x, y, z)),
                                    )))
                                    .expect("failed to send data");
                                self.info.loaded_entities.main_camera.set_field(
                                    field_name.to_string(),
                                    EntityField::Point3(Point3::new(x, y, z)),
                                );
                            }
                        }
                        EntityField::Angle(angle) => {
                            let mut x = angle.x;
                            ui.label("x");

                            ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                            let mut y = angle.y;
                            ui.label("y");
                            ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();
                            let mut z = angle.z;
                            ui.label("z");
                            ui.add(egui::DragValue::new(&mut z).speed(0.1)).changed();
                            if x != angle.x || y != angle.y || z != angle.z {
                                self.message_chanel
                                    .send(GuiPushMessage::SetCameraData((
                                        field_name.clone(),
                                        EntityField::Point3(Point3::new(x, y, z)),
                                    )))
                                    .expect("failed to send data");
                                self.info.loaded_entities.main_camera.set_field(
                                    field_name.to_string(),
                                    EntityField::Angle(Vector3::new(x, y, z)),
                                );
                            }
                        }
                    }
                }
            });
        ui.separator();
    }
    fn log_window(&mut self, ui: &mut egui::Ui) {
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
                            let data = msg.get_data();
                            let text =
                                egui::RichText::new(&data.data).color(egui::Rgba::from_gray(0.8));

                            let module_path = egui::RichText::new(match &data.module_path {
                                Some(d) => d.clone(),
                                None => "".to_string(),
                            })
                            .color(egui::Rgba::from_gray(0.8));
                            let log_level = match msg {
                                LogMessage::Trace(_) => egui::RichText::new("Trace")
                                    .color(egui::Rgba::from_rgb(0.3, 0.8, 0.3)),
                                LogMessage::Debug(_) => egui::RichText::new("Debug")
                                    .color(egui::Rgba::from_rgb(0.3, 1.0, 0.3)),
                                LogMessage::Info(_) => egui::RichText::new("Info")
                                    .color(egui::Rgba::from_rgb(0.3, 0.3, 1.0)),
                                LogMessage::Warn(_) => egui::RichText::new("Warn")
                                    .color(egui::Rgba::from_rgb(0.3, 0.1, 0.3)),
                                LogMessage::Error(_) => egui::RichText::new("Error")
                                    .color(egui::Rgba::from_rgb(1.0, 0.3, 0.3)),
                            };

                            ui.add(egui::Label::new(log_level));
                            ui.add(egui::Label::new(module_path));
                            ui.add(egui::Label::new(text));
                            ui.end_row();
                        }
                    });
            });
    }
}

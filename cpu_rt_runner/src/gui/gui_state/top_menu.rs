use super::GuiState;
use crate::messages::GuiPushMessage;
use lib_minya::ray_tracer::{CurrentShader, RayTracer};
use log::{error, info};
use std::str::FromStr;
impl GuiState {
    pub fn top_menu(&mut self, ui: &mut egui::Ui) {
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
                if ui.button("Save Scene").clicked() {
                    if let Some(save_path) = rfd::FileDialog::new()
                        .add_filter("extension", &[RayTracer::SCENE_FILE_EXTENSION])
                        .save_file()
                    {
                        if let Err(err) = self
                            .message_chanel
                            .send(GuiPushMessage::SaveScene(save_path))
                        {
                            error!(
                                "failed to save scene, rendering thread chrashed error: {:?}",
                                err
                            )
                        }
                    } else {
                        info!("save scene canceled")
                    }
                }
                if ui.button("Load Scene").clicked() {
                    if let Some(load_path) = rfd::FileDialog::new()
                        .add_filter("extension", &[RayTracer::SCENE_FILE_EXTENSION])
                        .pick_file()
                    {
                        if let Err(err) = self
                            .message_chanel
                            .send(GuiPushMessage::LoadScene(load_path))
                        {
                            error!(
                                "failed to load scene. Rendering thread crashed, error: {:#?}",
                                err
                            )
                        }
                    } else {
                        info!("scene load canceled")
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
                            .send(GuiPushMessage::SetShader(
                                CurrentShader::from_str(&s).unwrap(),
                            ))
                            .expect("failed to send");
                    }
                }
            });
        });
    }
}

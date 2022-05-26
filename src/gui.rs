use egui_miniquad as egui_mq;
pub struct GuiCtx {
    egui_mq: egui_mq::EguiMq,
}
impl GuiCtx {
    pub fn new(ctx: &mut miniquad::Context) -> Self {
        Self {
            egui_mq: egui_mq::EguiMq::new(ctx),
        }
    }
    pub fn draw(&mut self, ctx: &mut miniquad::Context) {
        self.egui_mq.run(ctx, |egui_ctx| {
            egui::Window::new("Hello world").show(egui_ctx, |ui| {});
            ()
        });
        self.egui_mq.draw(ctx);
    }
}

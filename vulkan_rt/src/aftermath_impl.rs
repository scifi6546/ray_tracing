use ash::vk;
use std::io::Write;
pub struct AftermathState {
    #[cfg(feature = "aftermath")]
    aftermath: aftermath_rs::Aftermath,
}
impl AftermathState {
    /// initiates state of nvidia aftermath handler
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "aftermath")]
            aftermath: aftermath_rs::Aftermath::new(FileAftermathDelegate),
        }
    }
    pub fn handle_error(&self, error: vk::Result) {
        #[cfg(feature = "aftermath")]
        {
            let status =
                aftermath_rs::Status::wait_for_status(Some(std::time::Duration::from_secs(5)));
            if status != aftermath_rs::Status::Finished {
                panic!("unexpected crash, status: {:#?}", status)
            }
        }
    }
}
#[cfg(feature = "aftermath")]
struct FileAftermathDelegate;

#[cfg(feature = "aftermath")]
impl aftermath_rs::AftermathDelegate for FileAftermathDelegate {
    fn dumped(&mut self, dump_data: &[u8]) {
        let mut file = std::fs::File::create("./crash_dump.nv-gpudmp").expect("failed to open");
        file.write_all(dump_data).expect("failed to write file");

        // Write `dump_data` to file, or send to telemetry server
    }
    fn shader_debug_info(&mut self, data: &[u8]) {}

    fn description(&mut self, describe: &mut aftermath_rs::DescriptionBuilder) {}
}

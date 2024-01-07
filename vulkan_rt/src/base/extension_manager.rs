use std::ffi::CStr;
use std::os::raw::c_char;
pub struct ExtensionManager {
    extensions: Vec<&'static CStr>,
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self { extensions: vec![] }
    }
    /// adds extension to manager.  if extension is not present it is not added
    pub fn add_extension(&mut self, extension: &'static CStr) {
        let found = self.extensions.contains(&extension);

        if !found {
            self.extensions.push(extension);
        }
    }
    pub unsafe fn add_extension_ptr(&mut self, extension_ptr: *const c_char) {
        let extension = CStr::from_ptr(extension_ptr);
        self.add_extension(extension)
    }
    pub fn extensions(&self) -> Vec<*const i8> {
        self.extensions
            .iter()
            .map(|extension| extension.as_ptr() as *const i8)
            .collect()
    }
    pub unsafe fn extensions_string(&self) -> Vec<String> {
        self.extensions
            .iter()
            .map(|name| name.to_str().unwrap().to_string())
            .collect()
    }
    /// check if extensions vec contains all extensions required
    pub fn contains(&self, extensions: &[String]) -> bool {
        self.extensions
            .iter()
            .map(|name| extensions.contains(&name.to_str().unwrap().to_string()))
            .fold(true, |acc, x| acc && x)
    }
    pub unsafe fn print(&self) {
        println!("extension count: {}", self.extensions.len());
        for name in self.extensions.iter() {
            println!("name: {}", name.to_str().unwrap());
        }
    }
}

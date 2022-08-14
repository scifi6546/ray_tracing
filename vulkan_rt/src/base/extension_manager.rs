use std::ffi::CStr;
use std::os::raw::c_char;
pub struct ExtensionManager {
    extensions: Vec<*const c_char>,
}
pub enum ContainError {
    DoesNotContain(String),
}
impl ExtensionManager {
    pub fn new() -> Self {
        Self { extensions: vec![] }
    }
    /// adds extension to manager.  if extension is not present it is not added
    pub unsafe fn add_extension(&mut self, extension: *const c_char) {
        let found = self
            .extensions
            .iter()
            .map(|self_e| strcmp(*self_e, extension))
            .fold(false, |acc, x| acc || x);
        if !found {
            self.extensions.push(extension);
        }
    }
    pub fn extensions(&self) -> &[*const c_char] {
        &self.extensions
    }
    pub unsafe fn extensions_string(&self) -> Vec<String> {
        self.extensions
            .iter()
            .map(|name| {
                CStr::from_ptr(*name)
                    .to_str()
                    .expect("failed to get str")
                    .to_string()
            })
            .collect()
    }
    /// check if extensions vec contains all extensions required
    pub fn contains(&self, extensions: &[String]) -> bool {
        self.extensions
            .iter()
            .map(|name| unsafe {
                CStr::from_ptr(*name)
                    .to_str()
                    .expect("failed to get str")
                    .to_string()
            })
            .map(|name| extensions.contains(&name))
            .fold(true, |acc, x| acc && x)
    }
    pub unsafe fn print(&self) {
        println!("extension count: {}", self.extensions.len());
        for name in self.extensions.iter() {
            let name_cstr = CStr::from_ptr(*name);
            let name_str = name_cstr
                .to_str()
                .expect("failed to convert extension name to string");
            println!("name: {}", name_str);
        }
    }
}
unsafe fn strcmp(a: *const c_char, b: *const c_char) -> bool {
    // should be enough bytes for everyone - probably bill gates
    const MAX_STR_LEN: isize = 255;
    for i in 0..MAX_STR_LEN {
        let a_char = *a.offset(i);
        let b_char = *b.offset(i);
        if a_char == 0 && b_char == 0 {
            return true;
        } else if a_char == 0 || b_char == 0 {
            return false;
        }
    }
    return false;
}

use std::os::raw::c_char;
pub struct ExtensionManager {
    extensions: Vec<*const c_char>,
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

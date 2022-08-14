extern crate core;
use ash::{extensions::ext::DebugUtils, vk, Entry};
use rfd::FileDialog;
use std::{
    borrow::Cow,
    ffi::{c_void, CStr, CString},
};
fn main() {
    unsafe { vk_main() }
}

unsafe fn vk_main() {
    let api_file = FileDialog::new().pick_file();
    let entry = match api_file {
        Some(path) => unsafe { Entry::load_from(path).expect("failed to load api") },
        None => Entry::linked(),
    };
    println!("supported layers");
    let mut layer_names: Vec<Option<String>> = vec![None];
    unsafe {
        let layers = entry
            .enumerate_instance_layer_properties()
            .expect("failed to get layers");
        for layer in layers.iter() {
            let name = CStr::from_ptr(layer.layer_name.as_ptr())
                .to_str()
                .expect("failed to convert to string")
                .to_string();

            println!("\t{}", name);
            layer_names.push(Some(name));
        }
        for layer in layer_names.iter() {
            let extension = layer
                .as_ref()
                .map(|l| unsafe { CStr::from_bytes_with_nul_unchecked(l.as_bytes()) });
            let extension_res = entry.enumerate_instance_extension_properties(extension);
            if extension_res.is_err() {
                println!("error for layer: {:?}", layer);
                continue;
            }
            let extensions = extension_res.ok().unwrap();
            println!("supported extensions for layer: {:?}: ", layer);
            for extension in extensions.iter() {
                let name = CStr::from_ptr(extension.extension_name.as_ptr())
                    .to_str()
                    .expect("failed to convert to string");
                println!("\t{}", name)
            }
        }
    }
    let app_info = vk::ApplicationInfo::builder().api_version(vk::make_api_version(0, 1, 3, 0));
    let layer_names = [CStr::from_bytes_with_nul_unchecked(
        b"VK_LAYER_KHRONOS_validation\0",
    )];
    let layer_names_raw = layer_names
        .iter()
        .map(|l| unsafe { l.as_ptr() })
        .collect::<Vec<_>>();
    let extension_names_raw = [DebugUtils::name().as_ptr()];
    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_names_raw)
        .enabled_extension_names(&extension_names_raw);
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback));
    let instance = entry
        .create_instance(&instance_create_info, None)
        .expect("failed to create instance");
    let debug_utils_loader = DebugUtils::new(&entry, &instance);
    let debug_callback = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .expect("failed to create debug callback")
    };
    let physical_devices = instance
        .enumerate_physical_devices()
        .expect("failed to get device");
    for dev in physical_devices.iter() {
        let prop = instance.get_physical_device_properties(*dev);
        let name_str = CStr::from_ptr(prop.device_name.as_ptr())
            .to_str()
            .expect("failed to get name");

        println!("device name: {}", name_str);
        let extension_properties = instance
            .enumerate_device_extension_properties(*dev)
            .expect("failed to get extension");
        for ext in extension_properties.iter() {
            let ext_name = CStr::from_ptr(ext.extension_name.as_ptr())
                .to_str()
                .expect("failed to get name");
            println!("\t{}", ext_name);
        }
    }
}
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );
    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        let bt = backtrace::Backtrace::new();
        println!("{:?}", bt);
        panic!()
    }

    vk::FALSE
}

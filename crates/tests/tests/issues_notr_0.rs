use std::{
    ffi::{c_char, c_void, CStr},
    ptr,
};

use cdump::{CDebug, CDeserialize, CSerialize};
use tests::{empty_deserializer, empty_serializer, empty_sizeof, eval_debug};

#[derive(CDebug, Clone, Copy, CDeserialize, CSerialize)]
#[repr(C)]
pub struct VkInstanceCreateInfo {
    #[cdump(dynamic(serializer = empty_serializer, deserializer = empty_deserializer, size_of = empty_sizeof))]
    pub p_next: *const c_void,
    pub p_application_info: *const VkApplicationInfo,
    pub enabled_layer_count: u32,
    #[cdump(array(len = self.enabled_layer_count))]
    pub pp_enabled_layer_names: *const *const c_char,
    pub enabled_extension_count: u32,
    #[cdump(array(len = self.enabled_extension_count))]
    pub pp_enabled_extension_names: *const *const c_char,
}

#[derive(CDebug, Clone, Copy, CDeserialize, CSerialize)]
#[repr(C)]
pub struct VkApplicationInfo {
    #[cdump(dynamic(serializer = empty_serializer, deserializer = empty_deserializer, size_of = empty_sizeof))]
    pub p_next: *const c_void,
    pub p_application_name: *const c_char,
    pub p_engine_name: *const c_char,
}

#[test]
fn vk_instance_create_info() {
    let text1 = c"vulkaninfo";
    let application_info = VkApplicationInfo {
        p_next: ptr::null(),
        p_application_name: text1.as_ptr(),
        p_engine_name: ptr::null(),
    };

    let text1 = c"VK_KHR_surface";
    let text2 = c"VK_KHR_win32_surface";
    let extensions = [text1.as_ptr(), text2.as_ptr()];
    let obj = VkInstanceCreateInfo {
        p_next: ptr::null(),
        p_application_info: &application_info,
        enabled_layer_count: 0,

        pp_enabled_layer_names: ptr::null(),
        enabled_extension_count: extensions.len() as u32,
        pp_enabled_extension_names: extensions.as_ptr(),
    };

    eval_debug(&obj);

    let mut buf = cdump::CDumpBufferWriter::new(16);
    unsafe { obj.serialize(&mut buf) };

    let mut reader = buf.into_reader();
    let copy = unsafe { VkInstanceCreateInfo::deserialize_ref(&mut reader) };

    eval_debug(&copy);

    assert_eq!(obj.p_next, copy.p_next);

    assert_ne!(obj.p_application_info, copy.p_application_info);
    let obj_ai = unsafe { *obj.p_application_info };
    let copy_ai = unsafe { *copy.p_application_info };
    assert_eq!(obj_ai.p_next, copy_ai.p_next);
    assert_ne!(obj_ai.p_application_name, copy_ai.p_application_name);
    assert_eq!(
        unsafe { CStr::from_ptr(obj_ai.p_application_name) },
        unsafe { CStr::from_ptr(copy_ai.p_application_name) },
    );
    assert_eq!(obj_ai.p_engine_name, copy_ai.p_engine_name);

    assert_eq!(obj.enabled_layer_count, copy.enabled_layer_count);
    assert_eq!(obj.pp_enabled_layer_names, copy.pp_enabled_layer_names);
    assert_eq!(obj.enabled_extension_count, copy.enabled_extension_count);
    assert_ne!(
        obj.pp_enabled_extension_names,
        copy.pp_enabled_extension_names
    );
    for i in 0..copy.enabled_extension_count {
        assert_eq!(
            unsafe { CStr::from_ptr(*obj.pp_enabled_extension_names.add(i as usize)) },
            unsafe { CStr::from_ptr(*copy.pp_enabled_extension_names.add(i as usize)) },
        );
    }
}

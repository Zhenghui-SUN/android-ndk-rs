//! Bindings for `ANativeWindow`
use std::ptr::NonNull;

use jni_sys::{jobject, JNIEnv};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NativeWindow {
    ptr: NonNull<ffi::ANativeWindow>,
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl NativeWindow {
    pub unsafe fn from_ptr(ptr: NonNull<ffi::ANativeWindow>) -> Self {
        Self { ptr }
    }

    pub unsafe fn from_surface(env: *mut JNIEnv, surface: jobject) -> Self {
        Self::from_ptr(NonNull::new(ffi::ANativeWindow_fromSurface(env as *mut ffi::JNIEnv, surface as _)).unwrap())
    }

    pub fn ptr(&self) -> NonNull<ffi::ANativeWindow> {
        self.ptr
    }
}

impl NativeWindow {
    pub fn height(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getHeight(self.ptr.as_ptr()) }
    }

    pub fn width(&self) -> i32 {
        unsafe { ffi::ANativeWindow_getWidth(self.ptr.as_ptr()) }
    }

    pub fn acquire(&self) {
        unsafe { ffi::ANativeWindow_acquire(self.ptr.as_ptr()); }
    }

    pub fn release(&self) {
        unsafe { ffi::ANativeWindow_release(self.ptr.as_ptr()); }
    }
    
    pub fn set_buffers_geometry(
        &self, 
        width: i32,
        height: i32,
        format: i32) {
        unsafe { 
            ffi::ANativeWindow_setBuffersGeometry(self.ptr.as_ptr(), width, height, format); 
        }
    }

    pub fn lock(
        &self, 
        out_buffer: *mut ffi::ANativeWindow_Buffer,
        in_out_dirty_bounds: *mut ffi::ARect) -> i32 {
        unsafe { ffi::ANativeWindow_lock(self.ptr.as_ptr(), out_buffer, in_out_dirty_bounds) }
    }

    pub fn unlock_and_post(&self) -> i32 {
        unsafe { ffi::ANativeWindow_unlockAndPost(self.ptr.as_ptr()) }
    }

    pub fn generate_epmty_buffer(width: i32, height: i32, stride: i32, format: i32) -> ffi::ANativeWindow_Buffer {
        ffi::ANativeWindow_Buffer { width, height, stride, format,
            bits: 0 as *mut std::os::raw::c_void, reserved: [0, 0, 0, 0, 0, 0] }
            // [0b00000000010101001011010101010100_u32; 720*1600].as_mut_ptr() as *mut std::os::raw::c_void
            // [132_u8; 720*1600*4].as_mut_ptr() as *mut std::os::raw::c_void
    }

    pub fn generate_empty_rect(top: i32, left: i32, right: i32, bottom: i32) -> ffi::ARect {
        ffi::ARect { top, left, right, bottom}
    }
}

#![cfg(target_os="android")]
#![allow(non_snake_case)]

mod apk;
mod error;
mod manifest;

pub use apk::ApkBuilder;
pub use error::Error;
pub use log::info;

use tokio::prelude::*;
use tokio::timer::Interval;
use std::time::{Duration, Instant};

use ndk::hardware_buffer::HardwareBufferFormat;
use ndk::native_window::NativeWindow;
use std::slice;

use rand::Rng;

use jni_sys::JNIEnv;
use jni::objects::{JClass, JString};
use jni_sys::{jstring, jboolean, jobject, jint, JNI_TRUE};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on", logger(level = "info", tag = "fatal")))]
pub fn main() {
    info!("hello world!");
    draw_color_timer();
}

#[no_mangle]
pub unsafe extern fn Java_com_example_rust_1demo_RustUtils_drawColor__Landroid_view_Surface_2I(env: *mut JNIEnv, _: JClass, surface: jobject, color: jint) -> jboolean {
    println!("call Java_com_example_rust_1demo_RustUtils_drawColor__Landroid_view_Surface_2I"); 
    SURFACE_NATIVE_WINDOW = Some(NativeWindow::from_surface(env, surface));
    drawColor(env, surface, color);
    0
}

static mut SURFACE_NATIVE_WINDOW: Option<NativeWindow> = None;

unsafe fn drawColor(env: *mut JNIEnv, surface: jobject, colorARGB: jint) { 
    // let alpha = colorARGB >> 24 & 0xFF; 
    // let red = colorARGB >> 16 & 0xFF; 
    // let green = colorARGB >> 8 & 0xFF; 
    // let blue = colorARGB & 0xFF; 
    // let colorABGR = alpha << 24 | (blue << 16) | (green << 8) | red; 
    // draw_on_native_window(&NativeWindow::from_surface(env, surface));
    draw_on_native_window(&(SURFACE_NATIVE_WINDOW.as_ref().unwrap()));
    // let task = Interval::new(Instant::now(), Duration::from_secs(3))
    // .take(100)
    // .for_each(|_instant| {
    //     draw_on_native_window(&(SURFACE_NATIVE_WINDOW.as_ref().unwrap()));
    //     Ok(())
    // })
    // .map_err(|e| panic!("interval errored; err={:?}", e));

    // tokio::run(task);
} 

fn draw_color_timer() {
    let task = Interval::new(Instant::now(), Duration::from_secs(1))
    .take(2)
    .for_each(|_instant| {
        let window = &*ndk_glue::native_window();
        info!("ndk_glue::native_window() = {:?}", window);
        match window {
            Some(w) => {
                draw_on_native_window(w);
            },
            _ => ()
        };
        Ok(())
    })
    .map_err(|e| panic!("interval errored; err={:?}", e));

    tokio::run(task);
}

fn draw_on_native_window(nativewindow: &NativeWindow) {
    unsafe { 
        info!("begin: {:?}", Instant::now());
        let h = nativewindow.height() as u32;
        let w = nativewindow.width() as u32;
        draw_rect_on_window(
            &nativewindow, 
            vec![rand::thread_rng().gen_range(0..255),
                rand::thread_rng().gen_range(0..255),
                rand::thread_rng().gen_range(0..255),
                rand::thread_rng().gen_range(0..255)
                ],
            ndk_glue::Rect {
                top: rand::thread_rng().gen_range(0..h/2), 
                left: rand::thread_rng().gen_range(0..w/2),
                bottom: rand::thread_rng().gen_range(h/2..h), 
                right: rand::thread_rng().gen_range(w/2..w)
            }
        );
        info!("end: {:?}", Instant::now());
    }
}

unsafe fn draw_rect_on_window(nativewindow: &NativeWindow, colors: Vec<u8>, rect: ndk_glue::Rect) {
    // let wi = &*native_window();
    // let nativewindow = match wi {
    //     Some(w) => w,
    //     _ => panic!("???") 
    // };
    // let nativewindow = NativeWindow::from_ptr(NonNull::new(window).unwrap());
    let height = nativewindow.height();
    let width = nativewindow.width();
    let color_format = get_color_format();
    let format = color_format.0;
    let bbp = color_format.1;
    // info!("set_window_color(): height = {}, width = {}, format = {}, bbp = {}, colors = {:?}, rect = {:?}", 
        // height, width, format, bbp, colors, rect);
    nativewindow.set_buffers_geometry(width, height, format);
    nativewindow.acquire();
    let mut buffer = NativeWindow::generate_epmty_buffer(width, height, width, format);
    let locked = nativewindow.lock(&mut buffer, &mut NativeWindow::generate_empty_rect(0, 0, width, height));
    // info!("locked: {}, format: {}", locked, buffer.format);
    if locked < 0 {
        nativewindow.release();
        return;
    }
    // *(buffer.bits as *mut [u32; width*height]) = [0b00000000010101001011010101010100_u32; width*height];
    // let src = [133; width*height];
    // let mut array = *(buffer.bits as *mut [u8; width*height]);
    // for i in (0..(width*height)) {
    //     array[i] = 133;
    // }
    draw_rect_into_buffer(buffer.bits, colors, rect, width, height);
    let blocks = slice::from_raw_parts(buffer.bits as *const u8, (width * height * bbp) as usize);
    info!("blocks len = {:?}", blocks.len());

    // 可run但是无效：
    // let mut line = buffer.bits;
    // let mut data = unsafe { slice::from_raw_parts(line as *const u32, 720 * 1600) };
    // data = &[0b00000000010101001011010101010100_u32; 720*1600]; 

    // for y in (0..(*buffer).height) { 
    //     for x in (0..(*buffer).width) { 
    //          line[x] = 151525191; 
    //         } 
    //     line += (*buffer).stride; 
    // } 
    let result = nativewindow.unlock_and_post();
    // info!("unlockAndPost result: {}", result);
    nativewindow.release();
    // info!("set_window_color() complete");
}


fn get_color_format() -> (i32, i32) {
    let format = HardwareBufferFormat::R8G8B8A8_UNORM;
    let bbp = match format {
        HardwareBufferFormat::R8G8B8A8_UNORM => 4,
        HardwareBufferFormat::R8G8B8X8_UNORM => 4,
        HardwareBufferFormat::R8G8B8_UNORM => 3,
        HardwareBufferFormat::R5G6B5_UNORM => 2,
        HardwareBufferFormat::R16G16B16A16_FLOAT => 8,
        HardwareBufferFormat::R10G10B10A2_UNORM => 4,
        HardwareBufferFormat::BLOB => 4,
        HardwareBufferFormat::D16_UNORM => 2,
        HardwareBufferFormat::D24_UNORM => 3,
        HardwareBufferFormat::D24_UNORM_S8_UINT => 3,
        HardwareBufferFormat::D32_FLOAT => 4,
        HardwareBufferFormat::D32_FLOAT_S8_UINT =>4,
        HardwareBufferFormat::S8_UINT => 1,
        HardwareBufferFormat::Y8Cb8Cr8_420 => 3,
    };
    (format as i32, bbp)
}

unsafe fn draw_rect_into_buffer(bits: *mut ::std::os::raw::c_void, colors: Vec<u8>, rect: ndk_glue::Rect, window_width: i32, window_height: i32) {
    let bbp = colors.len() as u32;
    // let width = rect.right - rect.left;
    // let height = rect.bottom - rect.top;
    let window_width = window_width as u32;
    for i in rect.top+1..=rect.bottom {
        for j in rect.left+1..=rect.right {
            let cur = (j + (i-1) * window_width - 1) * bbp;
            for k in 0..bbp {
                *(bits.offset((cur + (k as u32)) as isize) as *mut u8) = colors[k as usize];
            }
        }
    }
    // let mut i = 0;
    // while i < width*height*bbp {
    //     for j in 0..bbp {
    //         *(bits.offset((i+j) as isize) as *mut u8) = colors[j as usize];
    //     }
    //     i += bbp;
    // }
}

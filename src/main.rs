use std::{sync::Arc, fmt::Write};

use crt_term_gl::ScreenInfo;
use glfw::Context;
use glow::HasContext;

fn main() {
    let mut glfw = glfw::init::<()>(None).unwrap();
    let (mut win, events) = glfw
        .create_window(540, 300, "crt-term-gl", glfw::WindowMode::Windowed)
        .unwrap();

    let gl =
        Arc::new(unsafe { glow::Context::from_loader_function(|proc| win.get_proc_address(proc)) });

    let draw_size = win.get_framebuffer_size();
    let mut crt = crt_term_gl::CRTTerm::new(
        gl.clone(),
        ScreenInfo {
            gl_pos: [-1.0, -1.0],
            gl_size: [2.0, 2.0],
            //chars_size: [54, 15],
            chars_size: [20, 5],
            frame_size: [draw_size.0 as u32, draw_size.1 as u32]
        },
    );

    unsafe { gl.clear_color(1.0, 1.0, 1.0, 1.0) };
    win.make_current();
    win.set_framebuffer_size_polling(true);

    crt.write_str("\nHello, world. I hope you can hear me.\n\nMission Day 65535\n----------------").unwrap();
    let mut counter = 0;
    while !win.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(width, height) = event {
                unsafe { gl.viewport(0, 0, width, height) };
                crt.screen_changed(ScreenInfo {
                    gl_pos: [-1.0, -1.0],
                    gl_size: [2.0, 2.0],
                    // chars_size: [60, 15],
                    chars_size: [20, 5],
                    frame_size: [width as u32, height as u32]
                },);
            }
        }

        // counter += 1;
        // if counter >= 40 {
        //     counter = 0;
        // }

        // let c = match counter / 10 {
        //     0 => '|',
        //     1 => '/',
        //     2 => '-',
        //     3 => '\\',
        //     _ => unreachable!()
        // };

        // crt.cursor = [0, 1];
        // crt.write_char(c).unwrap();

        unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };

        crt.update();

        win.swap_buffers();
    }
}

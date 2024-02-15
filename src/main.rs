use std::{sync::Arc, fmt::Write};

use crt_term_gl::ScreenInfo;
use glfw::Context;
use glow::HasContext;


fn main() {
    let mut glfw = glfw::init::<()>(None).unwrap();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    let (mut win, events) = glfw
        .create_window(720, 405, "crt-term-gl", glfw::WindowMode::Windowed)
        .unwrap();

    let gl =
        Arc::new(unsafe { glow::Context::from_loader_function(|proc| win.get_proc_address(proc)) });

    let draw_size = win.get_framebuffer_size();

    let default_screen_info = ScreenInfo {
        gl_pos: [-1.0, -1.0],
        gl_size: [2.0, 2.0],
            
        chars_size: [74, 29],
        frame_size: [0; 2],

        back_color: [0x0a, 0x22, 0x16],
        color: [0x30, 0xff, 0x80],
    };

    let mut crt = crt_term_gl::CRTTerm::new(
        gl.clone(),
        ScreenInfo {
            frame_size: [draw_size.0 as u32, draw_size.1 as u32],
            ..default_screen_info
        },
    );

    unsafe { gl.clear_color(1.0, 1.0, 1.0, 1.0) };
    win.make_current();
    win.set_framebuffer_size_polling(true);

    let string = "\
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam justo justo, aliquet vestibulum egestas sit amet, iaculis id enim. Mauris ullamcorper, ipsum eget ultricies congue, velit lacus varius libero, sed ultricies ex dui a eros. Cras id urna malesuada, fermentum nibh vitae, lacinia libero. Aliquam dui dui, tempus at lectus quis, posuere blandit velit. In sed varius sem, sit amet gravida justo. Proin ut ex massa. Phasellus sed dui semper, mollis ex in, elementum metus. Aenean dapibus augue interdum ante scelerisque aliquam. Aliquam erat volutpat. Nullam eleifend venenatis erat aliquet pretium.\n\
\n\
Curabitur gravida urna ut ligula aliquet vehicula. Curabitur viverra tortor eget mauris vestibulum, et pulvinar justo dapibus. Phasellus sed magna at libero gravida condimentum malesuada at felis. Sed elementum eget lacus porttitor hendrerit. Vivamus sodales at massa id auctor. Donec sed metus nec lorem porttitor porta sit amet non felis. Nam cursus consequat purus, vel tristique ipsum vivamus.";
    let mut pos = 0;

    while !win.should_close() {
        glfw.poll_events();

        for (_, event) in glfw::flush_messages(&events) {
            if let glfw::WindowEvent::FramebufferSize(width, height) = event {
                unsafe { gl.viewport(0, 0, width, height) };
                crt.screen_changed(ScreenInfo {
                    frame_size: [width as u32, height as u32],
                    ..default_screen_info
                },);
            }
        }

        if pos < string.len() {
            let char = string[pos..].chars().next();
            if let Some(char) = char {
                let _ = crt.write_char(char);
                pos += char.len_utf8();
            }
        }

        unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };

        crt.update();

        win.swap_buffers();
    }
}

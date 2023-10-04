use std::{array, fmt::Write, fs::File, io::BufReader, sync::Arc, time::Instant};

use glow::HasContext;

#[derive(Debug, Clone, Copy)]
pub struct ScreenInfo {
    pub gl_pos: [f32; 2],
    pub gl_size: [f32; 2],
    pub frame_size: [u32; 2],
    pub chars_size: [usize; 2],
}

pub struct CRTTerm<C: HasContext> {
    gl: Arc<C>,

    screen: ScreenInfo,

    main_quad_buf: C::Buffer,
    full_quad_buf: C::Buffer,
    font_buf: C::Buffer,
    cursor_buf: C::Buffer,
    main_buf_verts: C::VertexArray,
    quad_buf_verts: C::VertexArray,
    font_buf_verts: C::VertexArray,
    cursor_buf_verts: C::VertexArray,
    default_program: C::Program,
    white_program: C::Program,
    crt_fading_program: C::Program,
    crt_warp_program: C::Program,
    font_texture: C::Texture,
    fade_texture: C::Texture,
    fade_framebuffer: C::Framebuffer,

    //time_uniform: C::UniformLocation,
    start_time: Instant,
    font_buffer_cache: Vec<u8>,
    font_buffer_vertices: u32,

    pub cursor: [usize; 2],
    cursor_blinker: i32, 
    chars: Box<[Box<[char]>]>,
}

const VERT_SHADER: &str = r#"#version 330 core
  layout (location = 0) in vec2 pos;
  layout (location = 1) in vec2 uv_in;

  out vec2 uv;
  void main() {
    gl_Position = vec4(pos.x, pos.y, 0.0, 1.0);
    uv = uv_in;
  }
"#;

const FRAG_SHADER: &str = r#"#version 330 core
  uniform sampler2D sampler;
  in vec2 uv;

  void main() {
    gl_FragColor = texture(sampler, uv);
  }
"#;

const WHITE_FRAG_SHADER: &str = r#"#version 330 core
  uniform sampler2D sampler;
  in vec2 uv;

  void main() {
    gl_FragColor = vec4(1.0);
  }
"#;

const CRT_WARP_FRAG_SHADER: &str = include_str!("crt_warp.frag.glsl");
const CRT_FADING_FRAG_SHADER: &str = include_str!("crt_fading.frag.glsl");

const FONT_5X11: &[u8] = include_bytes!("../font_5x11.png");
const FONT_COLS: u32 = 32;
const FONT_ROWS: u32 = 4;
const FONT_CHAR_WIDTH: u32 = 5;
const FONT_CHAR_HEIGHT: u32 = 11;
const FONT_IMAGE_SPACING_X: u32 = 1;
const FONT_IMAGE_SPACING_Y: u32 = 1;
const FONT_SPACING_X: u32 = 1;
const FONT_SPACING_Y: u32 = 1;

impl<C: HasContext> CRTTerm<C> {
    pub fn new(gl: Arc<C>, screen: ScreenInfo) -> Self {
        let font_image =
            image::load_from_memory_with_format(FONT_5X11, image::ImageFormat::Png).unwrap();
        let font_image = font_image.into_rgba8();

        let main_quad_buf = unsafe { gl.create_buffer().unwrap() };
        let full_quad_buf = unsafe { gl.create_buffer().unwrap() };
        let font_buf = unsafe { gl.create_buffer().unwrap() };
        let cursor_buf = unsafe { gl.create_buffer().unwrap() };

        let main_buf_verts = unsafe { gl.create_vertex_array().unwrap() };
        let quad_buf_verts = unsafe { gl.create_vertex_array().unwrap() };
        let font_buf_verts = unsafe { gl.create_vertex_array().unwrap() };
        let cursor_buf_verts = unsafe { gl.create_vertex_array().unwrap() };

        let default_program = unsafe { gl.create_program().unwrap() };
        let white_program = unsafe { gl.create_program().unwrap() };
        let crt_warp_program = unsafe { gl.create_program().unwrap() };
        let crt_fading_program = unsafe { gl.create_program().unwrap() };
        let font_texture = unsafe { gl.create_texture().unwrap() };

        let fade_texture = unsafe { gl.create_texture().unwrap() };
        let fade_framebuffer = unsafe { gl.create_framebuffer().unwrap() };

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(main_quad_buf));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                &create_quad_data_tri_strip(
                    screen.gl_pos,
                    screen.gl_size,
                    [0.0, 0.0],
                    [1.0, 1.0],
                    false,
                ),
                glow::STATIC_DRAW,
            );

            gl.bind_vertex_array(Some(main_buf_verts));
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(font_buf));
            gl.bind_vertex_array(Some(font_buf_verts));
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(cursor_buf));
            gl.bind_vertex_array(Some(cursor_buf_verts));
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(full_quad_buf));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                &create_quad_data_tri_strip(
                    [-1.0, -1.0],
                    [2.0, 2.0],
                    [0.0, 0.0],
                    [1.0, 1.0],
                    false,
                ),
                glow::STATIC_DRAW,
            );

            gl.bind_vertex_array(Some(quad_buf_verts));
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 16, 0);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, 16, 8);
            gl.enable_vertex_attrib_array(0);
            gl.enable_vertex_attrib_array(1);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            construct_program(
                gl.as_ref(),
                crt_warp_program,
                VERT_SHADER,
                CRT_WARP_FRAG_SHADER,
            );
            construct_program(
                gl.as_ref(),
                crt_fading_program,
                VERT_SHADER,
                CRT_FADING_FRAG_SHADER,
            );
            construct_program(gl.as_ref(), default_program, VERT_SHADER, FRAG_SHADER);
            construct_program(gl.as_ref(), white_program, VERT_SHADER, WHITE_FRAG_SHADER);

            gl.bind_texture(glow::TEXTURE_2D, Some(font_texture));
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                font_image.width() as i32,
                font_image.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(std::mem::transmute(font_image.as_ref())),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(fade_texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                screen.frame_size[0] as i32,
                screen.frame_size[1] as i32,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_R,
                glow::CLAMP_TO_BORDER as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_BORDER as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_BORDER as i32,
            );
            //gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fade_framebuffer));

            gl.framebuffer_texture(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                Some(fade_texture),
                0,
            );
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            // gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        //let time_uniform = unsafe { gl.get_uniform_location(crt_warp_program, "time") };

        Self {
            gl,
            screen,
            full_quad_buf,
            main_quad_buf,
            font_buf,
            cursor_buf,
            
            main_buf_verts,
            quad_buf_verts,
            font_buf_verts,
            cursor_buf_verts,

            default_program,
            white_program,
            crt_warp_program,
            crt_fading_program,

            font_texture,
            fade_framebuffer,
            fade_texture,

            //time_uniform,
            start_time: Instant::now(),
            font_buffer_cache: vec![],
            font_buffer_vertices: 0,

            cursor: [0, 0],
            cursor_blinker: 0,
            chars: (0..screen.chars_size[1])
                .map(|_| {
                    (0..screen.chars_size[0])
                        .map(|_| '\0')
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn update(&mut self) {
        self.font_buffer_cache.clear();
        self.font_buffer_vertices = 0;

        unsafe {
            let gl = &self.gl;
            gl.bind_vertex_array(Some(self.quad_buf_verts));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.full_quad_buf));
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fade_framebuffer));
            gl.framebuffer_texture(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                Some(self.fade_texture),
                0,
            );

            // let buf_w =
            //     (self.screen.chars_size[0] as u32 * (FONT_CHAR_WIDTH + FONT_SPACING_X)) as i32;
            // let buf_h =
            //     (self.screen.chars_size[1] as u32 * (FONT_CHAR_HEIGHT + FONT_SPACING_Y)) as i32;

            gl.bind_texture(glow::TEXTURE_2D, Some(self.fade_texture));
            gl.use_program(Some(self.crt_fading_program));
            gl.uniform_1_f32(
                gl.get_uniform_location(self.crt_fading_program, "time")
                    .as_ref(),
                Instant::now()
                    .saturating_duration_since(self.start_time)
                    .as_secs_f32(),
            );
            gl.uniform_2_f32(
                gl.get_uniform_location(self.crt_fading_program, "pixelSize")
                    .as_ref(),
                1.0 / self.screen.frame_size[0] as f32,
                1.0 / self.screen.frame_size[1] as f32,
            );
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            let char_bounds_w = self.screen.gl_size[0] / self.screen.chars_size[0] as f32;
            let char_bounds_h = self.screen.gl_size[1] / self.screen.chars_size[1] as f32;

            let char_w = (char_bounds_w / (FONT_CHAR_WIDTH + FONT_SPACING_X) as f32)
                * FONT_CHAR_WIDTH as f32;
            let char_h = (char_bounds_h / (FONT_CHAR_HEIGHT + FONT_SPACING_Y) as f32)
                * FONT_CHAR_HEIGHT as f32;

            for (y, row) in self.chars.iter().enumerate() {
                for (x, char) in row.iter().copied().enumerate() {

                    if char == '\0' {
                        continue;
                    }

                    let gl_x = self.screen.gl_pos[0] + x as f32 * char_bounds_w;
                    let gl_y = self.screen.gl_pos[1] + self.screen.gl_size[1]
                        - char_h
                        - y as f32 * char_bounds_h;

                    add_glyph(
                        &mut self.font_buffer_cache,
                        char,
                        [gl_x, gl_y],
                        [char_w, char_h],
                    );
                }
            }

            if self.cursor_blinker > 0 {
                let gl_x = self.screen.gl_pos[0] + self.cursor[0] as f32 * char_bounds_w;
                let gl_y = self.screen.gl_pos[1] + self.screen.gl_size[1]
                    - (self.cursor[1] + 1) as f32 * char_bounds_h;

                gl.bind_vertex_array(Some(self.cursor_buf_verts));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.cursor_buf));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    &create_quad_data_tri_strip([gl_x, gl_y], [char_bounds_w, char_bounds_h], [0.0, 0.0], [0.0, 0.0], false),
                    glow::STREAM_DRAW,
                );
                gl.use_program(Some(self.white_program));
                gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
            }

            gl.bind_vertex_array(Some(self.font_buf_verts));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.font_buf));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                &self.font_buffer_cache,
                glow::STREAM_DRAW,
            );
            gl.bind_texture(glow::TEXTURE_2D, Some(self.font_texture));
            gl.use_program(Some(self.default_program));
            
            gl.draw_arrays(glow::TRIANGLES, 0, self.font_buffer_cache.len() as i32 / 16);

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            gl.bind_vertex_array(Some(self.main_buf_verts));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.main_quad_buf));
            gl.use_program(Some(self.crt_warp_program));
            gl.uniform_1_f32(gl.get_uniform_location(self.crt_warp_program, "scanlineSize").as_ref(), 
            1.0 / ( self.screen.chars_size[1] as f32 * (FONT_CHAR_HEIGHT + FONT_SPACING_X) as f32)
            );
            gl.bind_texture(glow::TEXTURE_2D, Some(self.fade_texture));
            gl.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
        }

        self.cursor_blinker = match self.cursor_blinker {
            0 => -60,
            -1 => 60,
            b if b < 0 => b + 1,
            b => b - 1,
        };
    }

    pub fn screen_changed(&mut self, screen: ScreenInfo) {
        let gl = &self.gl;

        unsafe {
            if self.screen.gl_size != screen.gl_size || self.screen.gl_pos != screen.gl_pos {
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.main_quad_buf));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    &create_quad_data_tri_strip(
                        screen.gl_pos,
                        screen.gl_size,
                        [0.0, 0.0],
                        [1.0, 1.0],
                        false,
                    ),
                    glow::STATIC_DRAW,
                );
            }

            if self.screen.frame_size != screen.frame_size {
                gl.bind_texture(glow::TEXTURE_2D, Some(self.fade_texture));
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGB as i32,
                    screen.frame_size[0] as i32,
                    screen.frame_size[1] as i32,
                    0,
                    glow::RGB,
                    glow::UNSIGNED_BYTE,
                    None,
                );
            }

            if self.screen.chars_size != screen.chars_size {
                self.chars = (0..screen.chars_size[1])
                    .map(|y| {
                        (0..screen.chars_size[0])
                            .map(|x| {
                                self.chars
                                    .get(y)
                                    .and_then(|r| r.get(x))
                                    .copied()
                                    .unwrap_or('\0')
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice()
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
            }
        }

        self.screen = screen;
    }

    fn scroll(&mut self) {
        let mut buf = (0..self.screen.chars_size[0])
            .map(|_| '\0')
            .collect::<Vec<_>>()
            .into_boxed_slice();
        for i in 1..self.screen.chars_size[1] {
            buf.copy_from_slice(&self.chars[i]);
            self.chars[i - 1].copy_from_slice(&buf);
        }
        if let Some(last) = self.chars.last_mut() {
            for char in last.iter_mut() {
                *char = '\0'
            }
        }
    }
}

impl<C: HasContext> std::fmt::Write for CRTTerm<C> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let err = s.chars().find_map(|c| self.write_char(c).err());
        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        let mut cursor = self.cursor;
        if !c.is_control() {
            if let Some(row) = self.chars.get_mut(self.cursor[1]) {
                if let Some(sym) = row.get_mut(self.cursor[0]) {
                    *sym = c;
                }
            }
        }
        if c == '\n' || cursor[0] >= self.screen.chars_size[0] {
            cursor[0] = 0;
            cursor[1] += 1;
        } else {
            cursor[0] += 1;
        }
        if cursor[1] >= self.screen.chars_size[1] {
            cursor[0] = 0;
            cursor[1] = self.screen.chars_size[1].max(1) - 1;
            self.scroll();
        }
        self.cursor = cursor;

        Ok(())
    }
}

fn add_glyph(buf: &mut Vec<u8>, char: char, pos: [f32; 2], size: [f32; 2]) {
    let (uv_pos, uv_size) = get_font_glyph_uv(char);
    buf.extend_from_slice(&create_quad_data_tris(pos, size, uv_pos, uv_size, true));
}

fn calc_quad_vertices(
    pos: [f32; 2],
    size: [f32; 2],
    uv_pos: [f32; 2],
    uv_size: [f32; 2],
    flip_v: bool,
) -> [[f32; 4]; 4] {
    let x = pos[0];
    let y = pos[1];
    let w = size[0];
    let h = size[1];

    let r = x + w;
    let b = y + h;

    let [uv_l, uv_t] = uv_pos;
    let uv_r = uv_l + uv_size[0];
    let uv_b = uv_t + uv_size[1];

    let (uv_t, uv_b) = if flip_v { (uv_b, uv_t) } else { (uv_t, uv_b) };

    let tl = [x, y, uv_l, uv_t];
    let tr = [r, y, uv_r, uv_t];
    let bl = [x, b, uv_l, uv_b];
    let br = [r, b, uv_r, uv_b];

    [tl, tr, bl, br]
}

// x y u v
fn create_quad_data_tri_strip(
    pos: [f32; 2],
    size: [f32; 2],
    uv_pos: [f32; 2],
    uv_size: [f32; 2],
    flip_v: bool,
) -> [u8; 64] {
    unsafe { std::mem::transmute(calc_quad_vertices(pos, size, uv_pos, uv_size, flip_v)) }
}

fn create_quad_data_tris(
    pos: [f32; 2],
    size: [f32; 2],
    uv_pos: [f32; 2],
    uv_size: [f32; 2],
    flip_v: bool,
) -> [u8; 96] {
    let [tl, tr, bl, br] = calc_quad_vertices(pos, size, uv_pos, uv_size, flip_v);
    let verts = [tl, tr, bl, tr, bl, br];

    unsafe { std::mem::transmute(verts) }
}

fn create_shader<C: HasContext>(gl: &C, source: &str, ty: u32) -> C::Shader {
    unsafe {
        let shader = gl.create_shader(ty).unwrap();
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            panic!("Could not compile shader type {ty}: {log}");
        }

        shader
    }
}

fn construct_program<C: HasContext>(gl: &C, program: C::Program, vert: &str, frag: &str) {
    unsafe {
        gl.attach_shader(program, create_shader(gl, vert, glow::VERTEX_SHADER));
        gl.attach_shader(program, create_shader(gl, frag, glow::FRAGMENT_SHADER));
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            panic!("Could not link program: {log}");
        }
    }
}

/// returns: `([x, y], [w, h])`
fn get_font_glyph_uv(char: char) -> ([f32; 2], [f32; 2]) {
    const FONT_IMAGE_WIDTH: u32 = FONT_COLS * (FONT_CHAR_WIDTH + FONT_IMAGE_SPACING_X);
    const FONT_IMAGE_HEIGHT: u32 = FONT_ROWS * (FONT_CHAR_HEIGHT + FONT_IMAGE_SPACING_Y);

    const CHAR_UV_WIDTH: f32 = FONT_CHAR_WIDTH as f32 / FONT_IMAGE_WIDTH as f32;
    const CHAR_UV_HEIGHT: f32 = FONT_CHAR_HEIGHT as f32 / FONT_IMAGE_HEIGHT as f32;

    let ascii = if char.is_ascii() { char as u8 } else { 63 };

    let col = ascii as u32 % FONT_COLS;
    let row = ascii as u32 / FONT_COLS;

    let x = col * (FONT_CHAR_WIDTH + FONT_IMAGE_SPACING_X);
    let y = row * (FONT_CHAR_HEIGHT + FONT_IMAGE_SPACING_Y);

    (
        [
            x as f32 / FONT_IMAGE_WIDTH as f32,
            y as f32 / FONT_IMAGE_HEIGHT as f32,
        ],
        [CHAR_UV_WIDTH, CHAR_UV_HEIGHT],
    )
}

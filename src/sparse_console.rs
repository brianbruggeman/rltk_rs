use super::{Console, RGB, Font, Shader};
//use gl::types::*;
use std::ptr;
use std::mem;
use std::os::raw::c_void;
use gl::types::*;
use super::gl;

/// Internal storage structure for sparse tiles.
pub struct SparseTile {
    pub idx : usize,
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB
}

#[allow(non_snake_case)]
#[allow(dead_code)]
/// A sparse console. Rather than storing every cell on the screen, it stores just cells that have
/// data.
pub struct SparseConsole {
    pub width :u32,
    pub height: u32,

    // Private
    tiles: Vec<SparseTile>,
    is_dirty: bool,

    // GL Stuff
    vertex_buffer : Vec<f32>,
    index_buffer : Vec<i32>,
    VBO: u32,
    VAO: u32,
    EBO: u32
}

#[allow(dead_code)]
impl SparseConsole {
    #[allow(non_snake_case)]
    /// Initializes the console.
    pub fn init(width:u32, height: u32, gl : &gl::Gles2) -> Box<SparseConsole> {
        // Console backing init

        let (VBO, VAO, EBO) = SparseConsole::init_gl_for_console(gl);

        let new_console = SparseConsole{
            width: width, 
            height: height, 
            VBO: VBO,
            VAO: VAO,
            EBO: EBO,
            tiles: Vec::new(),
            is_dirty: true,
            vertex_buffer : Vec::new(),
            index_buffer : Vec::new()
        };

        Box::new(new_console)
    }

    #[allow(non_snake_case)]
    /// Initializes OpenGL for the sparse console.
    fn init_gl_for_console(gl : &gl::Gles2) -> (u32, u32, u32) {
        let (mut VBO, mut VAO, mut EBO) = (0, 0, 0);
        
        unsafe {
            // Generate buffers and arrays, as well as attributes.
            gl.GenVertexArrays(1, &mut VAO);
            gl.GenBuffers(1, &mut VBO);
            gl.GenBuffers(1, &mut EBO);

            gl.BindVertexArray(VAO);

            gl.BindBuffer(gl::ARRAY_BUFFER, VBO);

            let stride = 11 * mem::size_of::<GLfloat>() as GLsizei;
            // position attribute
            gl.VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl.EnableVertexAttribArray(0);
            // color attribute
            gl.VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<GLfloat>()) as *const c_void);
            gl.EnableVertexAttribArray(1);
             // bgcolor attribute
            gl.VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<GLfloat>()) as *const c_void);
            gl.EnableVertexAttribArray(2);
            // texture coord attribute
            gl.VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, stride, (9 * mem::size_of::<GLfloat>()) as *const c_void);
            gl.EnableVertexAttribArray(3);
        };
        
        (VBO, VAO, EBO)
    }

    /// Helper to push a point to the shader.
    fn push_point(vertex_buffer : &mut Vec<f32>, x:f32, y:f32, fg:RGB, bg:RGB, ux:f32, uy:f32) {
        vertex_buffer.push(x);
        vertex_buffer.push(y);
        vertex_buffer.push(0.0);
        vertex_buffer.push(fg.r);
        vertex_buffer.push(fg.g);
        vertex_buffer.push(fg.b);
        vertex_buffer.push(bg.r);
        vertex_buffer.push(bg.g);
        vertex_buffer.push(bg.b);
        vertex_buffer.push(ux);
        vertex_buffer.push(uy);
    }

    /// Helper to build vertices for the sparse grid.
    fn rebuild_vertices(&mut self, gl : &gl::Gles2) {
        if self.tiles.is_empty() { return; }

        self.vertex_buffer.clear();
        self.index_buffer.clear();

        let glyph_size_x : f32 = 1.0 / 16.0;
        let glyph_size_y : f32 = 1.0 / 16.0;

        let step_x : f32 = 2.0 / self.width as f32;
        let step_y : f32 = 2.0 / self.height as f32;

        let mut index_count : i32 = 0;
        for t in self.tiles.iter() {
            let x = t.idx % self.width as usize;
            let y = t.idx / self.width as usize;

            let screen_x = (step_x * x as f32) - 1.0;
            let screen_y = (step_y * y as f32) - 1.0;
            let fg = t.fg;
            let bg = t.bg;
            let glyph = t.glyph;
            let glyph_x = glyph % 16;
            let glyph_y = 16 - (glyph / 16);

            let glyph_left = glyph_x as f32 * glyph_size_x;
            let glyph_right = (glyph_x+1) as f32 * glyph_size_x;
            let glyph_top = glyph_y as f32 * glyph_size_y;
            let glyph_bottom = (glyph_y-1) as f32 * glyph_size_y;

            SparseConsole::push_point(&mut self.vertex_buffer, screen_x + step_x, screen_y + step_y, fg, bg, glyph_right, glyph_top);
            SparseConsole::push_point(&mut self.vertex_buffer, screen_x + step_x, screen_y, fg, bg, glyph_right, glyph_bottom);
            SparseConsole::push_point(&mut self.vertex_buffer, screen_x, screen_y, fg, bg, glyph_left, glyph_bottom);
            SparseConsole::push_point(&mut self.vertex_buffer, screen_x, screen_y + step_y, fg, bg, glyph_left, glyph_top);

            self.index_buffer.push(0 + index_count);
            self.index_buffer.push(1 + index_count);
            self.index_buffer.push(3 + index_count);
            self.index_buffer.push(1 + index_count);
            self.index_buffer.push(2 + index_count);
            self.index_buffer.push(3 + index_count);

            index_count += 4;
        }
                
        unsafe {
            gl.BindBuffer(gl::ARRAY_BUFFER, self.VBO);
            gl.BufferData(gl::ARRAY_BUFFER,
                        (self.vertex_buffer.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        &self.vertex_buffer[0] as *const f32 as *const c_void,
                        gl::STATIC_DRAW);

            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.EBO);
            gl.BufferData(gl::ELEMENT_ARRAY_BUFFER,
                        (self.index_buffer.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                        &self.index_buffer[0] as *const i32 as *const c_void,
                        gl::STATIC_DRAW);
        }
    }
}

impl Console for SparseConsole {
    /// If the console has changed, rebuild the vertex buffer.
    fn rebuild_if_dirty(&mut self, gl : &gl::Gles2) {
         if self.is_dirty {
            self.rebuild_vertices(gl);
            self.is_dirty = false;
        }
    }

    /// Draws the console to OpenGL.
    fn gl_draw(&mut self, font : &Font, shader : &Shader, gl : &gl::Gles2) {
        unsafe {
            // bind Texture
            font.bind_texture(gl);

            // render container
            shader.useProgram(gl);
            gl.BindVertexArray(self.VAO);
            gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.EBO);
            gl.BindBuffer(gl::ARRAY_BUFFER, self.VBO);
            gl.DrawElements(gl::TRIANGLES, (self.tiles.len() * 6) as i32, gl::UNSIGNED_INT, ptr::null());
        }
        self.is_dirty = false;
    }

    /// Translates x/y to an index entry. Not really useful.
    fn at(&self, x:i32, y:i32) -> usize {
        (((self.height-1 - y as u32) * self.width) + x as u32) as usize
    }

    /// Clear the screen.
    fn cls(&mut self) {
        self.is_dirty = true;
        self.tiles.clear();
    }

    /// Clear the screen. Since we don't HAVE a background, it doesn't use it.
    fn cls_bg(&mut self, _background : RGB) {
        self.is_dirty = true;
        self.tiles.clear();
    }

    /// Prints a string to an x/y position.
    fn print(&mut self, x:i32, y:i32, output:&str) {
        self.is_dirty = true;
        let mut idx = self.at(x, y);
        let text = output.to_string();

        let bytes = text.as_bytes();
        for i in 0..bytes.len() {
            self.tiles.push(SparseTile{
                idx: idx,
                glyph: bytes[i],
                fg: RGB::from_f32(1.0, 1.0, 1.0),
                bg: RGB::from_f32(0.0, 0.0, 0.0),
            });
            idx += 1;
        }
    }

    /// Prints a string to an x/y position, with foreground and background colors.
    fn print_color(&mut self, x:i32, y:i32, fg:RGB, bg:RGB, output:&str) {
        self.is_dirty = true;
        let mut idx = self.at(x, y);
        let text = output.to_string();

        let bytes = text.as_bytes();
        for i in 0..bytes.len() {
            self.tiles.push(SparseTile{
                idx: idx,
                glyph: bytes[i],
                fg: fg,
                bg: bg,
            });
            idx += 1;
        }
    }

    /// Sets a single cell in the console
    fn set(&mut self, x:i32, y:i32, fg:RGB, bg:RGB, glyph:u8) {
        let idx = self.at(x, y);
        self.tiles.push(SparseTile{
            idx: idx,
            glyph: glyph,
            fg: fg,
            bg: bg,
        });
    }

    /// Sets a single cell in the console's background
    fn set_bg(&mut self, x:i32, y:i32, bg:RGB) {
        let idx = self.at(x, y);
        self.tiles[idx].bg = bg;
    }

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_box(&mut self, sx:i32, sy:i32, width:i32, height:i32, fg: RGB, bg: RGB) {
        for y in sy .. sy + height {
            for x in sx .. sx + width {
                self.set(x, y, RGB::from_f32(1.0, 1.0, 1.0), RGB::from_f32(0.0, 0.0, 0.0), 32);
            }
        }

        self.set(sx, sy, fg, bg, 218);
        self.set(sx + width, sy, fg, bg, 191);
        self.set(sx, sy + height, fg, bg, 192);
        self.set(sx + width, sy + height, fg, bg, 217);
        for x in sx+1 .. sx + width {
            self.set(x, sy, fg, bg, 196);
            self.set(x, sy + height, fg, bg, 196);
        }
        for y in sy+1 .. sy + height {
            self.set(sx, y, fg, bg, 179);
            self.set(sx + width, y, fg, bg, 179);
        }
    }

    /// Draws a horizontal progress bar
    fn draw_bar_horizontal(&mut self, sx:i32, sy:i32, width:i32, n:i32, max:i32, fg:RGB, bg: RGB) {
        let percent = n as f32 / max as f32;
        let fill_width = (percent * width as f32) as i32;
        for x in 0..width {
            if x <= fill_width {
                self.set(sx + x, sy, fg, bg, 178);
            } else {
                self.set(sx + x, sy, fg, bg, 176);
            }
        }
    }

    /// Draws a vertical progress bar
    fn draw_bar_vertical(&mut self, sx:i32, sy:i32, height:i32, n:i32, max:i32, fg:RGB, bg: RGB) {
        let percent = n as f32 / max as f32;
        let fill_height = height - ((percent * height as f32) as i32);
        for y in 0..height {
            if y >= fill_height {
                self.set(sx, sy + y, fg, bg, 178);
            } else {
                self.set(sx, sy + y, fg, bg, 176);
            }
        }
    }

    fn print_centered(&mut self, y:i32, text:&str) {
        self.is_dirty = true;
        self.print((self.width as i32 / 2) - (text.to_string().len() as i32/2), y, text);
    }

    fn print_color_centered(&mut self, y:i32, fg:RGB, bg:RGB, text:&str) {
        self.is_dirty = true;
        self.print_color((self.width as i32 / 2) - (text.to_string().len() as i32/2), y, fg, bg, text);
    }
}
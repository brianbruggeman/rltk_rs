use super::{RGB, Font, Shader};
use super::gl;

/// The internal storage type for tiles in a simple console.
pub struct Tile {
    pub glyph: u8,
    pub fg: RGB,
    pub bg: RGB
}

/// Trait that must be implemented by console types.
pub trait Console {
    /// Check to see if the internal OpenGL representation needs to be rebuilt, and do so if required.
    fn rebuild_if_dirty(&mut self, gl : &gl::Gles2);

    /// Tells the console to draw itself via OpenGL.
    fn gl_draw(&mut self, font : &Font, shader : &Shader, gl : &gl::Gles2);

    /// Converts an x/y coordinate to a console index number.
    fn at(&self, x:i32, y:i32) -> usize;

    /// Clear the console.
    fn cls(&mut self);

    /// Clear the console to a set background color, if supported.
    fn cls_bg(&mut self, background : RGB);

    /// Print a string at the specified x/y coordinate.
    fn print(&mut self, x:i32, y:i32, output:&str);

    /// Print a string in color at the specified x/y coordinate, with specified foreground and background.
    fn print_color(&mut self, x:i32, y:i32, fg:RGB, bg:RGB, output:&str);

    /// Sets a single cell to a color/glyph combination.
    fn set(&mut self, x:i32, y:i32, fg:RGB, bg:RGB, glyph:u8);

    /// Sets a single cell's background color
    fn set_bg(&mut self, x:i32, y:i32, bg:RGB);

    /// Draws a box, starting at x/y with the extents width/height using CP437 line characters
    fn draw_box(&mut self, x:i32, y:i32, width:i32, height:i32, fg: RGB, bg: RGB);

    /// Draws a horizontal progress bar
    fn draw_bar_horizontal(&mut self, x:i32, y:i32, width:i32, n:i32, max:i32, fg:RGB, bg: RGB);

    /// Draws a vertical progress bar
    fn draw_bar_vertical(&mut self, x:i32, y:i32, height:i32, n:i32, max:i32, fg:RGB, bg: RGB);

    fn print_centered(&mut self, y:i32, text:&str);
    fn print_color_centered(&mut self, y:i32, fg:RGB, bg:RGB, text:&str);
}
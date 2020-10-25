use super::rects::RenderRect;
use super::shade::RectShaderProgram;
use crate::gl;
use crate::gl::types::*;
use crate::renderer::Error;
use alacritty_terminal::term::SizeInfo;

use std::mem::size_of;
use std::ptr;

enum InsertError {
    Full,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    // TODO these can certainly be i16
    x: f32,
    y: f32,
    color: Rgba,
}

#[derive(Debug)]
pub struct SolidRectRenderer {
    program: RectShaderProgram,

    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,

    indices: Vec<u16>,
    vertices: Vec<Vertex>,
}

impl SolidRectRenderer {
    pub fn new() -> Result<Self, Error> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }

        Ok(Self {
            program: RectShaderProgram::new()?,
            vao,
            vbo,
            ebo,
            indices: Vec::new(),
            vertices: Vec::new(),
        })
    }

    pub fn draw(&mut self, size_info: &SizeInfo, rects: Vec<RenderRect>) {
        if rects.is_empty() {
            return;
        }

        // Prepare common state
        unsafe {
            // Setup buffers
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);

            gl::UseProgram(self.program.id);

            // Position.
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (size_of::<Vertex>()) as _,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Color
            gl::VertexAttribPointer(
                1,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                (size_of::<Vertex>()) as _,
                offset_of!(Vertex, color) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Remove padding from viewport.
            gl::Viewport(0, 0, size_info.width as i32, size_info.height as i32);

            // Change blending strategy.
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::SRC_ALPHA, gl::ONE);
        }

        let center_x = size_info.width / 2.;
        let center_y = size_info.height / 2.;

        for rect in &rects {
            if let Err(InsertError::Full) = self.append_rect(center_x, center_y, rect) {
                self.draw_accumulated();
            }
        }

        self.draw_accumulated();

        // FIXME should we really do this here? Can we depend on next stage properly resetting its state?
        unsafe {
            // FIXME should we really do this here? Can we depend on next stage properly resetting its state?
            // Reset data and buffers.
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            // Deactivate rectangle program again.
            // Reset blending strategy.
            gl::Disable(gl::BLEND);
            gl::BlendFunc(gl::SRC_COLOR, gl::ONE_MINUS_SRC_COLOR);

            let padding_x = size_info.padding_x as i32;
            let padding_y = size_info.padding_y as i32;
            let width = size_info.width as i32;
            let height = size_info.height as i32;
            gl::Viewport(padding_x, padding_y, width - 2 * padding_x, height - 2 * padding_y);
        }
    }

    fn append_rect(
        &mut self,
        center_x: f32,
        center_y: f32,
        rect: &RenderRect,
    ) -> Result<(), InsertError> {
        let index = self.vertices.len();
        if index >= 65536 - 4 {
            return Err(InsertError::Full);
        }
        let index = index as u16;

        if rect.alpha <= 0. {
            return Ok(());
        }

        // Calculate rectangle position.
        let x = (rect.x - center_x) / center_x;
        let y = -(rect.y - center_y) / center_y;
        let width = rect.width / center_x;
        let height = rect.height / center_y;
        let color = Rgba {
            r: rect.color.r,
            g: rect.color.g,
            b: rect.color.b,
            a: (rect.alpha * 255.) as u8,
        };

        self.vertices.push(Vertex { x, y, color });
        self.vertices.push(Vertex { x, y: y - height, color });
        self.vertices.push(Vertex { x: x + width, y, color });
        self.vertices.push(Vertex { x: x + width, y: y - height, color });

        self.indices.push(index);
        self.indices.push(index + 1);
        self.indices.push(index + 2);

        self.indices.push(index + 2);
        self.indices.push(index + 3);
        self.indices.push(index + 1);

        Ok(())
    }

    fn draw_accumulated(&mut self) {
        if self.indices.is_empty() {
            return;
        }

        // Upload accumulated buffers
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<Vertex>()) as isize,
                self.vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<u16>()) as isize,
                self.indices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_SHORT,
                ptr::null(),
            );
        }

        self.indices.clear();
        self.vertices.clear();
    }
}

mod mesh;

use std::fs::File;

use glium::{implement_vertex, uniform, Display, Program, Surface};
use glium_text::{FontTexture, TextDisplay, TextSystem};
use nalgebra::{Matrix4, Point3, Vector3};

use mesh::Mesh;

const VERTEX_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 position;

    uniform mat4 matrix;
    uniform vec3 color;
    out vec3 in_color;

    void main() {
        gl_Position = matrix * vec4(position, 1.0);
        in_color = color;
    }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 in_color;
    out vec4 color;

    void main() {
        color = vec4(in_color, 1.0);
    }
"#;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 3],
}

implement_vertex!(Vertex, position);

pub struct Renderer {
    program: Program,
    text_system: TextSystem,
    font: FontTexture,
    sphere: Mesh,
}

impl Renderer {
    pub fn new(display: &Display) -> Self {
        let text_system = TextSystem::new(display);
        let font = FontTexture::new(display, File::open("DejaVuSans.ttf").unwrap(), 24).unwrap();

        Renderer {
            program: Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap(),
            text_system,
            font,
            sphere: Mesh::sphere(display),
        }
    }

    pub fn draw(&mut self, display: &Display, t: f32) {
        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.02, 1.0);

        let (width, height) = target.get_dimensions();
        let aspect = width as f32 / height as f32;
        let matrix = Matrix4::new_perspective(aspect, 45.0_f32.to_radians(), 0.1, 100.0)
            * Matrix4::look_at_rh(
                &Point3::new(
                    5.0 * (6.28 * t / 10.0).sin(),
                    3.0,
                    5.0 * (6.28 * t / 10.0).cos(),
                ),
                &Point3::new(0.0, 0.0, 0.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );

        let uniforms = uniform! {
            matrix: *matrix.as_ref(),
            color: [0.4_f32, 1.0, 0.4],
        };

        self.sphere
            .draw(&mut target, &self.program, &uniforms, &Default::default());

        target.finish().unwrap();
    }
}

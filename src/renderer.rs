mod mesh;

use std::fs::File;

use glium::{
    draw_parameters::DrawParameters, implement_vertex, index, uniform, Display, Frame, Program,
    Surface, VertexBuffer,
};
use glium_text::{FontTexture, TextDisplay, TextSystem};
use nalgebra::{Matrix4, Point3, Vector3, Vector4};

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

    pub fn draw(&mut self, display: &Display, t: f32, satellites: Vec<(u8, Vector3<f64>)>) {
        let mut target = display.draw();

        target.clear_color(0.0, 0.0, 0.02, 1.0);

        let (width, height) = target.get_dimensions();
        let aspect = width as f32 / height as f32;
        let period = 20.0;
        let dist = 80e6;
        let matrix = Matrix4::new_perspective(aspect, 45.0_f32.to_radians(), 1000.0, 1e9)
            * Matrix4::look_at_rh(
                &Point3::new(
                    dist * (6.28 * t / period).sin(),
                    dist / 2.0,
                    dist * (6.28 * t / period).cos(),
                ),
                &Point3::new(0.0, 0.0, 0.0),
                &Vector3::new(0.0, 1.0, 0.0),
            );

        let uniforms = uniform! {
            matrix: *(matrix.prepend_scaling(6371000.0)).as_ref(),
            color: [0.4_f32, 1.0, 0.4],
        };

        self.sphere
            .draw(&mut target, &self.program, &uniforms, &Default::default());

        let scale = 5e5;
        let sat_vertex_buffer = VertexBuffer::new(
            display,
            &vec![
                Vertex {
                    position: [-scale, 0.0, 0.0],
                },
                Vertex {
                    position: [scale, 0.0, 0.0],
                },
                Vertex {
                    position: [0.0, -scale, 0.0],
                },
                Vertex {
                    position: [0.0, scale, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0, -scale],
                },
                Vertex {
                    position: [0.0, 0.0, scale],
                },
            ],
        )
        .unwrap();
        let sat_index_buffer = index::NoIndices(index::PrimitiveType::LinesList);

        for (sv_id, position) in satellites {
            let pos = Vector3::new(position.x as f32, position.y as f32, position.z as f32);
            let matrix = matrix.prepend_translation(&pos);
            let uniforms = uniform! {
                matrix: *matrix.as_ref(),
                color: [0.0_f32, 0.8, 1.0],
            };

            target
                .draw(
                    &sat_vertex_buffer,
                    &sat_index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();

            let satellite_pos = matrix * Vector4::new(0.0, 0.0, 0.0, 1.0);
            let matrix = Matrix4::<f32>::identity()
                .prepend_translation(&Vector3::new(
                    satellite_pos.x / satellite_pos.w,
                    satellite_pos.y / satellite_pos.w,
                    0.0,
                ))
                .prepend_nonuniform_scaling(&Vector3::new(1.0 / 40.0 / aspect, 1.0 / 40.0, 1.0));

            let label = format!("{}", sv_id);

            self.draw_text(&mut target, &label, matrix, Default::default());
        }

        target.finish().unwrap();
    }

    fn draw_text(
        &self,
        target: &mut Frame,
        text: &str,
        matrix: Matrix4<f32>,
        draw_parameters: DrawParameters,
    ) {
        let text = TextDisplay::new(&self.text_system, &self.font, text);

        glium_text::draw(
            &text,
            &self.text_system,
            target,
            *matrix.as_ref(),
            (1.0, 1.0, 1.0, 1.0),
            draw_parameters.clone(),
        );
    }
}

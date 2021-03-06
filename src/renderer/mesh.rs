use std::{collections::HashMap, f32::consts::PI};

use glium::{
    index, uniforms::Uniforms, Display, DrawParameters, Frame, IndexBuffer, Program, Surface,
    VertexBuffer,
};

use super::Vertex;

pub struct Mesh {
    vertices: VertexBuffer<Vertex>,
    indices: Vec<IndexBuffer<u32>>,
}

impl Mesh {
    pub fn sphere(display: &Display) -> Mesh {
        let n_meridians = 24;
        let n_parallels = 12;
        let n_subdivisions = 10;

        let mut vertices = vec![];
        let mut indices = vec![];

        // A vertex deduplication map. This ensures that all vertices are stored exactly once.
        // Points on the sphere are defined as (lat_index, lon_index) for the purpose of
        // non-duplication. lat_index is `0..=n_parallels * n_subdivisions`, lon_index is
        // `0..n_meridians * n_subdivisions`. The map maps point coordinates to vertex index.
        let mut result: HashMap<(u32, u32), u32> = HashMap::new();

        // generate parallels
        for parallel_index in 0..n_parallels + 1 {
            let mut parallel_indices = vec![];
            // inclusive range to append longitude 0 once more at the end of the index buffer
            for lon_index in 0..=n_meridians * n_subdivisions {
                let lat_index = parallel_index * n_subdivisions;
                // if we're at the end of the range, this will map lon_index to 0 again
                let lon_index = lon_index % (n_meridians * n_subdivisions);
                let key = (lat_index, lon_index);
                let entry = result.entry(key).or_insert_with(|| {
                    let lat = PI / ((n_parallels * n_subdivisions) as f32) * lat_index as f32;
                    let lon = 2.0 * PI / ((n_meridians * n_subdivisions) as f32) * lon_index as f32;
                    vertices.push(Vertex {
                        position: [lat.sin() * lon.cos(), lat.cos(), lat.sin() * lon.sin()],
                    });
                    vertices.len() as u32 - 1
                });
                // for poles, only insert lon_index = 0
                if lat_index == 0 || lat_index == n_parallels * n_subdivisions {
                    break;
                }
                parallel_indices.push(*entry);
            }
            if parallel_index == 0 || parallel_index == n_parallels {
                continue;
            }
            indices.push(
                IndexBuffer::new(display, index::PrimitiveType::LineStrip, &parallel_indices)
                    .unwrap(),
            );
        }

        for meridian_index in 0..n_meridians {
            let mut meridian_indices = vec![];
            for lat_index in 0..=n_parallels * n_subdivisions {
                let lon_index = if lat_index == 0 || lat_index == n_parallels * n_subdivisions {
                    0 // poles are always (lat, 0)
                } else {
                    meridian_index * n_subdivisions
                };
                let key = (lat_index, lon_index);
                let entry = result.entry(key).or_insert_with(|| {
                    let lat = PI / ((n_parallels * n_subdivisions) as f32) * lat_index as f32;
                    let lon = 2.0 * PI / ((n_meridians * n_subdivisions) as f32) * lon_index as f32;
                    vertices.push(Vertex {
                        position: [lat.sin() * lon.cos(), lat.cos(), lat.sin() * lon.sin()],
                    });
                    vertices.len() as u32 - 1
                });
                meridian_indices.push(*entry);
            }
            indices.push(
                IndexBuffer::new(display, index::PrimitiveType::LineStrip, &meridian_indices)
                    .unwrap(),
            );
        }

        let vertices = VertexBuffer::new(display, &vertices).unwrap();

        Mesh { vertices, indices }
    }

    pub fn draw<U: Uniforms>(
        &self,
        target: &mut Frame,
        program: &Program,
        uniforms: &U,
        draw_parameters: &DrawParameters,
    ) {
        for index_buffer in &self.indices {
            target
                .draw(
                    &self.vertices,
                    index_buffer,
                    program,
                    uniforms,
                    draw_parameters,
                )
                .unwrap();
        }
    }
}

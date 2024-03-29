use crate::*;
use emath::*;

/// The vertex type.
///
/// Should be friendly to send to GPU as is.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    /// Logical pixel coordinates (points).
    /// (0,0) is the top left corner of the screen.
    pub pos: Pos2, // 64 bit

    /// Normalized texture coordinates.
    /// (0, 0) is the top left corner of the texture.
    /// (1, 1) is the bottom right corner of the texture.
    pub uv: Pos2, // 64 bit

    /// sRGBA with premultiplied alpha
    pub color: Color32, // 32 bit
}

/// Textured triangles.
#[derive(Clone, Debug, Default)]
pub struct Triangles {
    /// Draw as triangles (i.e. the length is always multiple of three).
    pub indices: Vec<u32>,

    /// The vertex data indexed by `indices`.
    pub vertices: Vec<Vertex>,

    /// The texture to use when drawing these triangles
    pub texture_id: TextureId,
}

impl Triangles {
    pub fn with_texture(texture_id: TextureId) -> Self {
        Self {
            texture_id,
            ..Default::default()
        }
    }

    pub fn bytes_used(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.vertices.len() * std::mem::size_of::<Vertex>()
            + self.indices.len() * std::mem::size_of::<u32>()
    }

    /// Are all indices within the bounds of the contained vertices?
    pub fn is_valid(&self) -> bool {
        let n = self.vertices.len() as u32;
        self.indices.iter().all(|&i| i < n)
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() && self.vertices.is_empty()
    }

    /// Append all the indices and vertices of `other` to `self`.
    pub fn append(&mut self, other: Triangles) {
        debug_assert!(other.is_valid());

        if self.is_empty() {
            *self = other;
        } else {
            assert_eq!(
                self.texture_id, other.texture_id,
                "Can't merge Triangles using different textures"
            );

            let index_offset = self.vertices.len() as u32;
            for index in &other.indices {
                self.indices.push(index_offset + index);
            }
            self.vertices.extend(other.vertices.iter());
        }
    }

    pub fn colored_vertex(&mut self, pos: Pos2, color: Color32) {
        debug_assert!(self.texture_id == TextureId::Egui);
        self.vertices.push(Vertex {
            pos,
            uv: WHITE_UV,
            color,
        });
    }

    /// Add a triangle.
    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
    }

    /// Make room for this many additional triangles (will reserve 3x as many indices).
    /// See also `reserve_vertices`.
    pub fn reserve_triangles(&mut self, additional_triangles: usize) {
        self.indices.reserve(3 * additional_triangles);
    }

    /// Make room for this many additional vertices.
    /// See also `reserve_triangles`.
    pub fn reserve_vertices(&mut self, additional: usize) {
        self.vertices.reserve(additional);
    }

    /// Rectangle with a texture and color.
    pub fn add_rect_with_uv(&mut self, pos: Rect, uv: Rect, color: Color32) {
        #![allow(clippy::identity_op)]

        let idx = self.vertices.len() as u32;
        self.add_triangle(idx + 0, idx + 1, idx + 2);
        self.add_triangle(idx + 2, idx + 1, idx + 3);

        let right_top = Vertex {
            pos: pos.right_top(),
            uv: uv.right_top(),
            color,
        };
        let left_top = Vertex {
            pos: pos.left_top(),
            uv: uv.left_top(),
            color,
        };
        let left_bottom = Vertex {
            pos: pos.left_bottom(),
            uv: uv.left_bottom(),
            color,
        };
        let right_bottom = Vertex {
            pos: pos.right_bottom(),
            uv: uv.right_bottom(),
            color,
        };
        self.vertices.push(left_top);
        self.vertices.push(right_top);
        self.vertices.push(left_bottom);
        self.vertices.push(right_bottom);
    }

    /// Uniformly colored rectangle.
    pub fn add_colored_rect(&mut self, rect: Rect, color: Color32) {
        debug_assert!(self.texture_id == TextureId::Egui);
        self.add_rect_with_uv(rect, [WHITE_UV, WHITE_UV].into(), color)
    }

    /// This is for platforms that only support 16-bit index buffers.
    ///
    /// Splits this mesh into many smaller meshes (if needed).
    /// All the returned meshes will have indices that fit into a `u16`.
    pub fn split_to_u16(self) -> Vec<Triangles> {
        const MAX_SIZE: u32 = 1 << 16;

        if self.vertices.len() < MAX_SIZE as usize {
            return vec![self]; // Common-case optimization
        }

        let mut output = vec![];
        let mut index_cursor = 0;

        while index_cursor < self.indices.len() {
            let span_start = index_cursor;
            let mut min_vindex = self.indices[index_cursor];
            let mut max_vindex = self.indices[index_cursor];

            while index_cursor < self.indices.len() {
                let (mut new_min, mut new_max) = (min_vindex, max_vindex);
                for i in 0..3 {
                    let idx = self.indices[index_cursor + i];
                    new_min = new_min.min(idx);
                    new_max = new_max.max(idx);
                }

                if new_max - new_min < MAX_SIZE {
                    // Triangle fits
                    min_vindex = new_min;
                    max_vindex = new_max;
                    index_cursor += 3;
                } else {
                    break;
                }
            }

            assert!(
                index_cursor > span_start,
                "One triangle spanned more than {} vertices",
                MAX_SIZE
            );

            output.push(Triangles {
                indices: self.indices[span_start..index_cursor]
                    .iter()
                    .map(|vi| vi - min_vindex)
                    .collect(),
                vertices: self.vertices[(min_vindex as usize)..=(max_vindex as usize)].to_vec(),
                texture_id: self.texture_id,
            });
        }
        output
    }

    /// Translate location by this much, in-place
    pub fn translate(&mut self, delta: Vec2) {
        for v in &mut self.vertices {
            v.pos += delta;
        }
    }
}

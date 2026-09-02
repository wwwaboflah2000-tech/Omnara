mod voxel;

use godot::prelude::*;
use godot::classes::{MeshInstance3D, ArrayMesh, mesh::PrimitiveType};
use voxel::{SubChunk, CHUNK_SIZE};

struct OmnaraExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OmnaraExtension {}

// الاتجاهات الستة لفحص الجيران (Directions)
const DIRECTIONS: [[i32; 3]; 6] = [
    [0, 1, 0],   // Top (+Y)
    [0, -1, 0],  // Bottom (-Y)
    [0, 0, 1],   // South (+Z)
    [0, 0, -1],  // North (-Z)
    [1, 0, 0],   // East (+X)
    [-1, 0, 0],  // West (-X)
];

// نقاط الوجوه الستة (Face Vertices)
const FACE_VERTICES: [[[f32; 3]; 4]; 6] = [
    // Top (+Y)
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // Bottom (-Y)
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // South (+Z)
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // North (-Z)
    [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
    // East (+X)
    [[1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
    // West (-X)
    [[0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 1.0], [0.0, 1.0, 0.0]],
];

#[derive(GodotClass)]
#[class(base=MeshInstance3D)]
pub struct OmnaraChunkNode {
    base: Base<MeshInstance3D>,
}

#[godot_api]
impl IMeshInstance3D for OmnaraChunkNode {
    fn init(base: Base<MeshInstance3D>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("🔨 [OMNARA]: Generating SubChunk Mesh...");

        let mut sub_chunk = SubChunk::new();
        sub_chunk.generate_test_terrain();

        let mesh = self.build_mesh(&sub_chunk);
        self.base_mut().set_mesh(&mesh);
        
        godot_print!("✅ [OMNARA]: SubChunk Mesh Rendered Successfully!");
    }
}

impl OmnaraChunkNode {
    fn build_mesh(&self, chunk: &SubChunk) -> Gd<ArrayMesh> {
        let mut vertices = PackedVector3Array::new();
        let mut indices = PackedInt32Array::new();
        let mut vertex_count: i32 = 0;

        for y in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                for x in 0..CHUNK_SIZE as i32 {
                    let block = chunk.get_block(x, y, z);
                    if !block.is_opaque() {
                        continue; // تخطي الهواء
                    }

                    // فحص الاتجاهات الستة
                    for face_idx in 0..6 {
                        let dir = DIRECTIONS[face_idx];
                        let neighbor = chunk.get_block(x + dir[0], y + dir[1], z + dir[2]);

                        // إخفاء الأوجه: ارسم الوجه فقط إذا كان الجار غير مصمت
                        if !neighbor.is_opaque() {
                            let face = FACE_VERTICES[face_idx];
                            for v in face {
                                vertices.push(Vector3::new(
                                    x as f32 + v[0],
                                    y as f32 + v[1],
                                    z as f32 + v[2],
                                ));
                            }

                            // إضافة مثلثين لتكوين المربع (Indices)
                            indices.push(vertex_count);
                            indices.push(vertex_count + 1);
                            indices.push(vertex_count + 2);

                            indices.push(vertex_count);
                            indices.push(vertex_count + 2);
                            indices.push(vertex_count + 3);

                            vertex_count += 4;
                        }
                    }
                }
            }
        }

        let mut arrays = godot::classes::ArrayMesh::new_gd();
        if vertices.len() > 0 {
            let mut surface_arrays = godot::builtin::VariantArray::new();
            surface_arrays.resize(godot::classes::mesh::ArrayType::MAX.ord() as usize);
            
            surface_arrays.set(godot::classes::mesh::ArrayType::VERTEX.ord() as usize, &vertices.to_variant());
            surface_arrays.set(godot::classes::mesh::ArrayType::INDEX.ord() as usize, &indices.to_variant());

            arrays.add_surface_from_arrays(PrimitiveType::TRIANGLES, &surface_arrays);
        }

        arrays
    }
}

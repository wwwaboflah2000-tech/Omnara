mod voxel;

use godot::prelude::*;
use godot::classes::{MeshInstance3D, IMeshInstance3D, ArrayMesh, StandardMaterial3D, mesh::ArrayType, mesh::PrimitiveType};
use godot::builtin::VarArray;
use voxel::{SubChunk, BlockId, CHUNK_SIZE};

struct OmnaraExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OmnaraExtension {}

const DIRECTIONS: [[i32; 3]; 6] = [
    [0, 1, 0],   // 0: Top (+Y)
    [0, -1, 0],  // 1: Bottom (-Y)
    [0, 0, 1],   // 2: South (+Z)
    [0, 0, -1],  // 3: North (-Z)
    [1, 0, 0],   // 4: East (+X)
    [-1, 0, 0],  // 5: West (-X)
];

const FACE_NORMALS: [[f32; 3]; 6] = [
    [0.0, 1.0, 0.0],   // Top
    [0.0, -1.0, 0.0],  // Bottom
    [0.0, 0.0, 1.0],   // South
    [0.0, 0.0, -1.0],  // North
    [1.0, 0.0, 0.0],   // East
    [-1.0, 0.0, 0.0],  // West
];

// ⚡ تصحيح ترتيب النقاط (Counter-Clockwise CCW) لجميع الوجوه ⚡
const FACE_VERTICES: [[[f32; 3]; 4]; 6] = [
    // Top (+Y)
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // Bottom (-Y)
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // South (+Z)
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // North (-Z)
    [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
    // East (+X) - تم التصحيح هنا
    [[1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0]],
    // West (-X) - تم التصحيح هنا
    [[0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 1.0]],
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
        godot_print!("🔨 [OMNARA]: Generating Solid SubChunk with Fixed Face Culling...");

        let mut sub_chunk = SubChunk::new();
        sub_chunk.generate_test_terrain();

        let mesh = self.build_mesh(&sub_chunk);
        
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_vertex_color_use_as_albedo(true);

        self.base_mut().set_mesh(&mesh);
        self.base_mut().set_material_override(&mat);
        
        godot_print!("✅ [OMNARA]: Solid SubChunk Rendered Correctly!");
    }
}

impl OmnaraChunkNode {
    fn get_block_color(block: BlockId, face_idx: usize) -> Color {
        match block {
            BlockId::GRASS => {
                if face_idx == 0 {
                    Color::from_rgb(0.2, 0.8, 0.2) // عشب أخضر من الأعلى
                } else if face_idx == 1 {
                    Color::from_rgb(0.45, 0.3, 0.15) // تراب من الأسفل
                } else {
                    Color::from_rgb(0.35, 0.6, 0.2) // جانب العشب
                }
            }
            BlockId::DIRT => Color::from_rgb(0.5, 0.32, 0.15),
            BlockId::STONE => Color::from_rgb(0.55, 0.55, 0.55),
            _ => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    fn build_mesh(&self, chunk: &SubChunk) -> Gd<ArrayMesh> {
        let mut vertices = PackedVector3Array::new();
        let mut normals = PackedVector3Array::new();
        let mut colors = PackedColorArray::new();
        let mut indices = PackedInt32Array::new();
        let mut vertex_count: i32 = 0;

        for y in 0..CHUNK_SIZE as i32 {
            for z in 0..CHUNK_SIZE as i32 {
                for x in 0..CHUNK_SIZE as i32 {
                    let block = chunk.get_block(x, y, z);
                    if !block.is_opaque() {
                        continue;
                    }

                    for face_idx in 0..6 {
                        let dir = DIRECTIONS[face_idx];
                        let neighbor = chunk.get_block(x + dir[0], y + dir[1], z + dir[2]);

                        // إخفاء الأوجه الداخلية فقط ورسم الأوجه الخارجية
                        if !neighbor.is_opaque() {
                            let face = FACE_VERTICES[face_idx];
                            let normal = FACE_NORMALS[face_idx];
                            let color = Self::get_block_color(block, face_idx);

                            for v in face {
                                vertices.push(Vector3::new(
                                    x as f32 + v[0],
                                    y as f32 + v[1],
                                    z as f32 + v[2],
                                ));
                                normals.push(Vector3::new(normal[0], normal[1], normal[2]));
                                colors.push(color);
                            }

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

        let mut arrays = ArrayMesh::new_gd();
        if vertices.len() > 0 {
            let mut surface_arrays = VarArray::new();
            surface_arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
            
            surface_arrays.set(ArrayType::VERTEX.ord() as usize, &vertices.to_variant());
            surface_arrays.set(ArrayType::NORMAL.ord() as usize, &normals.to_variant());
            surface_arrays.set(ArrayType::COLOR.ord() as usize, &colors.to_variant());
            surface_arrays.set(ArrayType::INDEX.ord() as usize, &indices.to_variant());

            arrays.add_surface_from_arrays(PrimitiveType::TRIANGLES, &surface_arrays);
        }

        arrays
    }
}

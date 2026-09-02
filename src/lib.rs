mod voxel;

use godot::prelude::*;
use godot::classes::{
    MeshInstance3D, IMeshInstance3D, ArrayMesh, StandardMaterial3D,
    base_material_3d::Flags, base_material_3d::CullMode,
    mesh::ArrayType, mesh::PrimitiveType
};
use godot::builtin::VarArray;
use voxel::{SubChunk, BlockId, CHUNK_SIZE};

struct OmnaraExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OmnaraExtension {}

// 1. الاتجاهات الستة لفحص الجيران
const DIRECTIONS: [[i32; 3]; 6] = [
    [0, 1, 0],   // 0: Top (+Y)
    [0, -1, 0],  // 1: Bottom (-Y)
    [0, 0, 1],   // 2: South (+Z)
    [0, 0, -1],  // 3: North (-Z)
    [1, 0, 0],   // 4: East (+X)
    [-1, 0, 0],  // 5: West (-X)
];

// 2. متجهات الإضاءة لكل وجه
const FACE_NORMALS: [[f32; 3]; 6] = [
    [0.0, 1.0, 0.0],   // Top
    [0.0, -1.0, 0.0],  // Bottom
    [0.0, 0.0, 1.0],   // South
    [0.0, 0.0, -1.0],  // North
    [1.0, 0.0, 0.0],   // East
    [-1.0, 0.0, 0.0],  // West
];

// 3. مصفوفة نقاط الأوجه
const FACE_VERTICES: [[[f32; 3]; 4]; 6] = [
    // 0: Top (+Y)
    [[0.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
    // 1: Bottom (-Y)
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
    // 2: South (+Z)
    [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]],
    // 3: North (-Z)
    [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
    // 4: East (+X)
    [[1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0]],
    // 5: West (-X)
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
        godot_print!("🔨 [OMNARA]: Generating Optimized SubChunk (Zero Garbage)...");

        let mut sub_chunk = SubChunk::new();
        sub_chunk.generate_test_terrain();

        let mesh = self.build_mesh(&sub_chunk);
        
        // خامة معطل بها الـ Culling وتعتمد على ألوان النقاط
        let mut mat = StandardMaterial3D::new_gd();
        mat.set_flag(Flags::ALBEDO_FROM_VERTEX_COLOR, true);
        mat.set_cull_mode(CullMode::DISABLED);

        self.base_mut().set_mesh(&mesh);
        self.base_mut().set_material_override(&mat);
        
        godot_print!("✅ [OMNARA]: Optimized SubChunk Rendered Cleanly!");
    }
}

impl OmnaraChunkNode {
    fn get_block_color(block: BlockId, face_idx: usize) -> Color {
        match block {
            BlockId::GRASS => {
                if face_idx == 0 {
                    Color::from_rgb(0.2, 0.8, 0.2) // عشب أخضر في القمة
                } else if face_idx == 1 {
                    Color::from_rgb(0.45, 0.3, 0.15) // تراب في الأسفل
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
        // حجز مصفوفات Rust الأصلية السريعة مسبقاً لمنع إعادة الحجز المتكرر
        let mut rust_vertices = Vec::with_capacity(2000);
        let mut rust_normals = Vec::with_capacity(2000);
        let mut rust_colors = Vec::with_capacity(2000);
        let mut rust_indices = Vec::with_capacity(3000);
        
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

                        // ارسم الوجه فقط إذا كان الجار غير مصمت
                        if !neighbor.is_opaque() {
                            let face = FACE_VERTICES[face_idx];
                            let normal = FACE_NORMALS[face_idx];
                            let color = Self::get_block_color(block, face_idx);

                            for v in face {
                                rust_vertices.push(Vector3::new(
                                    x as f32 + v[0],
                                    y as f32 + v[1],
                                    z as f32 + v[2],
                                ));
                                rust_normals.push(Vector3::new(normal[0], normal[1], normal[2]));
                                rust_colors.push(color);
                            }

                            rust_indices.push(vertex_count);
                            rust_indices.push(vertex_count + 1);
                            rust_indices.push(vertex_count + 2);

                            rust_indices.push(vertex_count);
                            rust_indices.push(vertex_count + 2);
                            rust_indices.push(vertex_count + 3);

                            vertex_count += 4;
                        }
                    }
                }
            }
        }

        let mut arrays = ArrayMesh::new_gd();
        
        if !rust_vertices.is_empty() {
            // تحويل مصفوفات Rust إلى مصفوفات Godot دفعة واحدة
            let vertices_packed: PackedVector3Array = rust_vertices.into_iter().collect();
            let normals_packed: PackedVector3Array = rust_normals.into_iter().collect();
            let colors_packed: PackedColorArray = rust_colors.into_iter().collect();
            let indices_packed: PackedInt32Array = rust_indices.into_iter().collect();

            let mut surface_arrays = VarArray::new();
            surface_arrays.resize(ArrayType::MAX.ord() as usize, &Variant::nil());
            
            surface_arrays.set(ArrayType::VERTEX.ord() as usize, &vertices_packed.to_variant());
            surface_arrays.set(ArrayType::NORMAL.ord() as usize, &normals_packed.to_variant());
            surface_arrays.set(ArrayType::COLOR.ord() as usize, &colors_packed.to_variant());
            surface_arrays.set(ArrayType::INDEX.ord() as usize, &indices_packed.to_variant());

            arrays.add_surface_from_arrays(PrimitiveType::TRIANGLES, &surface_arrays);
        }

        arrays
    }
}

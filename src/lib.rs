mod voxel;
mod world;

use godot::prelude::*;
use godot::classes::{
    MeshInstance3D, IMeshInstance3D, ArrayMesh, StandardMaterial3D,
    base_material_3d::Flags, base_material_3d::CullMode,
    mesh::ArrayType, mesh::PrimitiveType
};
use godot::builtin::VarArray;
use voxel::{BlockId, CHUNK_SIZE, MIN_WORLD_Y, MAX_WORLD_Y};
use world::VoxelWorld;

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
        godot_print!("🔨 [OMNARA]: Generating 3x3 Full Landscape (Sea Level Y=63)...");

        let mut world = VoxelWorld::new();
        // توليد عالم متصل بمحيط 1 (أي 9 Chunks = 48x48 بلوكة)
        world.generate_test_world(1);

        // بناء مجسم العالم المتصل بالكامل
        let mesh = self.build_full_world_mesh(&world, 1);

        let mut mat = StandardMaterial3D::new_gd();
        mat.set_flag(Flags::ALBEDO_FROM_VERTEX_COLOR, true);
        mat.set_cull_mode(CullMode::DISABLED);

        self.base_mut().set_mesh(&mesh);
        self.base_mut().set_material_override(&mat);

        godot_print!("✅ [OMNARA]: 3x3 Landscape with Water & Bedrock Rendered Successfully!");
    }
}

impl OmnaraChunkNode {
    fn get_block_color(block: BlockId, face_idx: usize) -> Color {
        match block {
            BlockId::BEDROCK => Color::from_rgb(0.12, 0.12, 0.12), // بيدروك أسود
            BlockId::GRASS => {
                if face_idx == 0 {
                    Color::from_rgb(0.2, 0.8, 0.2) // عشب أخضر في القمة
                } else if face_idx == 1 {
                    Color::from_rgb(0.45, 0.3, 0.15)
                } else {
                    Color::from_rgb(0.35, 0.6, 0.2)
                }
            }
            BlockId::DIRT => Color::from_rgb(0.5, 0.32, 0.15),
            BlockId::STONE => Color::from_rgb(0.55, 0.55, 0.55),
            BlockId::SAND => Color::from_rgb(0.85, 0.82, 0.55),     // شاطئ رملي
            BlockId::WATER => Color::from_rgba(0.15, 0.45, 0.9, 0.8), // مياه البحر 63
            _ => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    // بناء مجسم العالم لجميع الـ 9 Chunks المتصلة
    fn build_full_world_mesh(&self, world: &VoxelWorld, radius: i32) -> Gd<ArrayMesh> {
        let mut rust_vertices = Vec::with_capacity(30000);
        let mut rust_normals = Vec::with_capacity(30000);
        let mut rust_colors = Vec::with_capacity(30000);
        let mut rust_indices = Vec::with_capacity(45000);

        let mut vertex_count: i32 = 0;

        let min_coord = -radius * CHUNK_SIZE as i32;
        let max_coord = (radius + 1) * CHUNK_SIZE as i32;

        for gx in min_coord..max_coord {
            for gz in min_coord..max_coord {
                // مسح الارتفاع من قاع البيدروك (-64) إلى سقف العالم
                for gy in MIN_WORLD_Y..MAX_WORLD_Y {
                    let block = world.get_block_global(gx, gy, gz);
                    if block == BlockId::AIR {
                        continue;
                    }

                    for face_idx in 0..6 {
                        let dir = DIRECTIONS[face_idx];
                        let neighbor = world.get_block_global(gx + dir[0], gy + dir[1], gz + dir[2]);

                        // رسم الوجه إذا كان الجار هواء أو ماء
                        let should_draw = if block == BlockId::WATER {
                            neighbor == BlockId::AIR
                        } else {
                            !neighbor.is_opaque()
                        };

                        if should_draw {
                            let face = FACE_VERTICES[face_idx];
                            let normal = FACE_NORMALS[face_idx];
                            let color = Self::get_block_color(block, face_idx);

                            for v in face {
                                rust_vertices.push(Vector3::new(
                                    gx as f32 + v[0],
                                    gy as f32 + v[1],
                                    gz as f32 + v[2],
                                ));
                                rust_normals.push(Vector3::new(normal[0], normal[1], normal[2]));
                                rust_colors.push(color);
                            }

                            indices_builder(&mut rust_indices, &mut vertex_count);
                        }
                    }
                }
            }
        }

        let mut arrays = ArrayMesh::new_gd();

        if !rust_vertices.is_empty() {
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

#[inline(always)]
fn indices_builder(indices: &mut Vec<i32>, vertex_count: &mut i32) {
    indices.push(*vertex_count);
    indices.push(*vertex_count + 1);
    indices.push(*vertex_count + 2);

    indices.push(*vertex_count);
    indices.push(*vertex_count + 2);
    indices.push(*vertex_count + 3);

    *vertex_count += 4;
}

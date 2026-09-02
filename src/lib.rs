use godot::prelude::*;

// 1. تسجيل الإضافة في Godot
struct OmnaraExtension;

#[gdextension]
unsafe impl ExtensionLibrary for OmnaraExtension {}

// 2. إنشاء أول عقدة (Node) تظهر في محرك Godot
#[derive(GodotClass)]
#[class(base=Node)]
pub struct OmnaraEngine {
    base: Base<Node>,
}

#[godot_api]
impl INode for OmnaraEngine {
    fn init(base: Base<Node>) -> Self {
        godot_print!("🌌 [OMNARA]: Rust Voxel Engine Initialized Successfully on Android!");
        Self { base }
    }
}

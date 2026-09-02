use godot::prelude::*;
use godot::classes::{
    CharacterBody3D, ICharacterBody3D, Camera3D, Node3D,
    CollisionShape3D, CapsuleShape3D, Input, InputEvent,
    InputEventMouseMotion, InputEventScreenDrag
};

const WALK_SPEED: f32 = 5.0;
const SPRINT_SPEED: f32 = 8.0;
const JUMP_VELOCITY: f32 = 7.0;
const GRAVITY: f32 = 20.0;
const ACCELERATION: f32 = 12.0;
const FRICTION: f32 = 14.0;
const MOUSE_SENSITIVITY: f32 = 0.003;
const TOUCH_SENSITIVITY: f32 = 0.005;
const BOB_FREQUENCY: f32 = 2.4;
const BOB_AMPLITUDE: f32 = 0.06;

#[inline(always)]
fn move_toward(from: f32, to: f32, delta: f32) -> f32 {
    if (to - from).abs() <= delta {
        to
    } else {
        from + (to - from).signum() * delta
    }
}

#[derive(GodotClass)]
#[class(base=CharacterBody3D)]
pub struct OmnaraPlayer {
    base: Base<CharacterBody3D>,
    head: Option<Gd<Node3D>>,
    camera: Option<Gd<Camera3D>>,
    head_rotation_x: f32,
    bob_timer: f32,
}

#[godot_api]
impl ICharacterBody3D for OmnaraPlayer {
    fn init(base: Base<CharacterBody3D>) -> Self {
        Self {
            base,
            head: None,
            camera: None,
            head_rotation_x: 0.0,
            bob_timer: 0.0,
        }
    }

    fn ready(&mut self) {
        let mut shape = CapsuleShape3D::new_gd();
        shape.set_radius(0.35);
        shape.set_height(1.8);

        let mut col_shape_node = CollisionShape3D::new_alloc();
        col_shape_node.set_shape(&shape);
        col_shape_node.set_position(Vector3::new(0.0, 0.9, 0.0));
        self.base_mut().add_child(&col_shape_node);

        let mut head_node = Node3D::new_alloc();
        head_node.set_position(Vector3::new(0.0, 1.6, 0.0));

        let mut cam_node = Camera3D::new_alloc();
        // ⚡ أمر ضروري جداً لمنع الشاشة السوداء: تفعيل الكاميرا فوراً ⚡
        cam_node.make_current(); 
        head_node.add_child(&cam_node);

        self.head = Some(head_node.clone());
        self.camera = Some(cam_node);

        self.base_mut().add_child(&head_node);
        
        // تعديل مكان النزول ليكون فوق أعلى قمة في التضاريس لتجنب السقوط في الفراغ
        self.base_mut().set_position(Vector3::new(8.0, 90.0, 8.0));
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_motion) = event.clone().try_cast::<InputEventMouseMotion>() {
            let rel = mouse_motion.get_relative();
            self.rotate_camera(rel.x, rel.y, MOUSE_SENSITIVITY);
        }

        if let Ok(touch_drag) = event.try_cast::<InputEventScreenDrag>() {
            let rel = touch_drag.get_relative();
            self.rotate_camera(rel.x, rel.y, TOUCH_SENSITIVITY);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let delta_f = delta as f32;
        let mut velocity = self.base().get_velocity();
        let is_on_floor = self.base().is_on_floor();

        if !is_on_floor {
            velocity.y -= GRAVITY * delta_f;
        }

        let input = Input::singleton();
        
        // ⚡ التغليف الآمن الصارم للنصوص لمنع انهيار المترجم واللعبة ⚡
        let action_jump = StringName::from("ui_accept");
        if input.is_action_just_pressed(&action_jump) && is_on_floor {
            velocity.y = JUMP_VELOCITY;
        }

        let input_vec = input.get_vector(
            &StringName::from("ui_left"),
            &StringName::from("ui_right"),
            &StringName::from("ui_up"),
            &StringName::from("ui_down"),
        );

        let speed = WALK_SPEED;
        let global_transform = self.base().get_global_transform();
        let forward = -global_transform.basis.col_c().normalized();
        let right = global_transform.basis.col_a().normalized();

        let move_dir = (forward * -input_vec.y + right * input_vec.x).normalized();

        if move_dir.length_squared() > 0.001 {
            velocity.x = move_toward(velocity.x, move_dir.x * speed, ACCELERATION * speed * delta_f);
            velocity.z = move_toward(velocity.z, move_dir.z * speed, ACCELERATION * speed * delta_f);

            if is_on_floor {
                self.bob_timer += delta_f * (speed * BOB_FREQUENCY);
                let bob_y = 1.6 + (self.bob_timer.sin() * BOB_AMPLITUDE);
                if let Some(head) = &mut self.head {
                    head.set_position(Vector3::new(0.0, bob_y, 0.0));
                }
            }
        } else {
            velocity.x = move_toward(velocity.x, 0.0, FRICTION * delta_f * 10.0);
            velocity.z = move_toward(velocity.z, 0.0, FRICTION * delta_f * 10.0);

            if let Some(head) = &mut self.head {
                let current_y = head.get_position().y;
                let target_y = move_toward(current_y, 1.6, delta_f * 2.0);
                head.set_position(Vector3::new(0.0, target_y, 0.0));
            }
        }

        self.base_mut().set_velocity(velocity);
        self.base_mut().move_and_slide();
    }
}

impl OmnaraPlayer {
    fn rotate_camera(&mut self, rel_x: f32, rel_y: f32, sens: f32) {
        self.base_mut().rotate_y(-rel_x * sens);

        self.head_rotation_x = (self.head_rotation_x - rel_y * sens).clamp(-1.55, 1.55);

        if let Some(head) = &mut self.head {
            head.set_rotation(Vector3::new(self.head_rotation_x, 0.0, 0.0));
        }
    }
}

use instance::D2ColorInstance;
use jandering_engine::{
    bind_group::camera::d2::D2CameraBindGroup,
    engine::EngineDescriptor,
    object::VertexRaw,
    shader::ShaderDescriptor,
    types::{UVec2, Vec2, Vec3},
};
use wasm_bindgen::prelude::*;

mod instance;

const RAPIER_TO_WORLD: f32 = 50.0;

#[wasm_bindgen(start)]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Coultn init");
    let mut engine = jandering_engine::engine::Engine::new(EngineDescriptor::default());

    let mut camera_bind_group = D2CameraBindGroup::new(&engine.renderer, true);
    camera_bind_group.right_click_move = true;
    let camera_bg = engine.renderer.add_bind_group(camera_bind_group);

    let untyped_bind_groups = [camera_bg.into()];
    let shader = jandering_engine::shader::create_shader(
        &mut engine.renderer,
        ShaderDescriptor {
            code: include_str!("shader.wgsl"),
            descriptors: &[VertexRaw::desc(), D2ColorInstance::desc()],
            bind_groups: &untyped_bind_groups,
            backface_culling: false,
            ..Default::default()
        },
    );

    let mut circles = jandering_engine::object::primitives::quad(
        &engine.renderer,
        vec![
            D2ColorInstance {
                scale: Vec2::new(100.0, 100.0),
                color: Vec3::new(0.0, 0.7, 0.5),
                ..Default::default()
            },
            D2ColorInstance {
                scale: Vec2::new(100.0, 100.0),
                color: Vec3::new(1.0, 0.7, 0.5),
                ..Default::default()
            },
        ],
    );

    let mut rigid_body_set = rapier2d::dynamics::RigidBodySet::new();
    let mut collider_set = rapier2d::geometry::ColliderSet::new();

    let collider = rapier2d::geometry::ColliderBuilder::cuboid(100.0, 0.1)
        .translation(Vec2::new(0.0, -8.1).into())
        .build();
    collider_set.insert(collider);

    let first_cube_handle = {
        /* Create the bouncing ball. */
        let rigid_body = rapier2d::dynamics::RigidBodyBuilder::dynamic()
            .translation(Vec2::new(0.0, 2.0).into())
            .build();
        let collider = rapier2d::geometry::ColliderBuilder::cuboid(1.0, 1.0)
            .restitution(1.0)
            .build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);
        ball_body_handle
    };
    let second_cube_handle = {
        /* Create the bouncing ball. */
        let rigid_body = rapier2d::dynamics::RigidBodyBuilder::dynamic()
            .translation(Vec2::new(0.5, 4.0).into())
            .build();
        let collider = rapier2d::geometry::ColliderBuilder::cuboid(1.0, 1.0)
            .restitution(1.0)
            .build();
        let ball_body_handle = rigid_body_set.insert(rigid_body);
        collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);
        ball_body_handle
    };
    /* Create other structures necessary for the simulation. */
    let gravity = rapier2d::na::vector![0.0, -9.81];
    let integration_parameters = rapier2d::dynamics::IntegrationParameters::default();
    let mut physics_pipeline = rapier2d::pipeline::PhysicsPipeline::new();
    let mut island_manager = rapier2d::dynamics::IslandManager::new();
    let mut broad_phase = rapier2d::geometry::BroadPhase::new();
    let mut narrow_phase = rapier2d::geometry::NarrowPhase::new();
    let mut impulse_joint_set = rapier2d::dynamics::ImpulseJointSet::new();
    let mut multibody_joint_set = rapier2d::dynamics::MultibodyJointSet::new();
    let mut ccd_solver = rapier2d::dynamics::CCDSolver::new();
    let mut query_pipeline = rapier2d::pipeline::QueryPipeline::new();

    engine.run(move |context, renderer| {
        let resolution = UVec2::new(renderer.config.width, renderer.config.height);
        let camera_bind_group = renderer.get_bind_group_t_mut(camera_bg).unwrap();
        camera_bind_group.update(context, resolution);

        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            Some(&mut query_pipeline),
            &(),
            &(),
        );
        {
            let body = &rigid_body_set[first_cube_handle];
            circles.instances[0].position = (body.translation() * RAPIER_TO_WORLD).into();
            circles.instances[0].rotation = body.rotation().angle();
        }
        {
            let body = &rigid_body_set[second_cube_handle];
            circles.instances[1].position = (body.translation() * RAPIER_TO_WORLD).into();
            circles.instances[1].rotation = body.rotation().angle();
        }

        circles.update(context, renderer);

        renderer.render(&[&circles], context, &shader, &untyped_bind_groups);
    });
}

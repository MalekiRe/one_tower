mod main_game;

use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_mod_picking::backend::PointerHits;
use bevy_mod_picking::backends::raycast::bevy_mod_raycast::prelude::{Ray3d, Raycast};
use bevy_mod_picking::backends::raycast::{RaycastBackendSettings, RaycastPickable};
use bevy_mod_picking::prelude::{Move, On, Pickable, Pointer, PointerId, PointerLocation};
use bevy_mod_picking::DefaultPickingPlugins;
use random_number::random;
use std::ops::{Add, Div, Mul};
use bevy_egui::{EguiContext, EguiContexts, EguiPlugin};
use bevy_xpbd_3d::plugins::PhysicsPlugins;
use bevy_xpbd_3d::prelude::{AngularVelocity, Collider, ExternalImpulse, Friction, Gravity, LinearVelocity, RigidBody};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_enemies)
        .add_systems(Update, move_enemy_to_mouse)
        .insert_resource(MousePos(Vec3::new(0.0, 0.0, 0.0)))
        .add_systems(Update, set_mouse_pos)
        .add_systems(Update, die)
        .add_systems(Update, ui_test)
        .run();
}

fn ui_test(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("top")
        .resizable(true)
        .show(ctx, |ui| {
            let _ = ui.button("hi");
        });
}

#[derive(Component)]
pub struct Tower;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    MainMenu,
    Staging,
    InGame,
}
/*
Game Idea

So you have a certain health level. When an enemy touches your mouse you lose 1 health, you do not regain health over the course of the game?

Enemies will follow your mouse around, but if your mouse gets too far from them then they will attack your towers

You can place towers if you have enough gold

You can also duplicate the enmies by shift clicking near them, but half will lose half their speed, but the other half will double their speed.
(this is good for trying to farm extra gold? )

When you die you can use the extra gold you had leftover and it converts in a 1,000:1 ratio to diamonds.
Also for every 1,000 enemies you kill you get 1 diamond.

At the start you get some amount of gold per enemy kill.

You cannot kill eneimies directly.

Only your towers can do it, you place towers by left clicking them, and you upgrade them by shift left clicking.

The enemies never stop coming.


 */

#[derive(Component)]
pub struct Speed(f32);

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub speed: Speed,
    pub pbr_bundle: PbrBundle,
    pub rigid_body: RigidBody,
    pub angular_velocity: AngularVelocity,
    pub collider: Collider,
    pub friction: Friction,
}

fn die(enemies: Query<&Transform, With<Enemy>>, mouse: ResMut<MousePos>) {
    let mut mouse = mouse.0;
    mouse.y = 0.0;
    for enemy in enemies.iter() {
        let mut t = enemy.translation;
        t.y = 0.0;
        if mouse.distance(t) < 0.03 {
            println!("dead");
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut time: Res<Time>,
) {
    let val: f32 = random!();
    if val > time.elapsed_seconds().log2() / 500.0{
        return;
    }
    let mut x: f32 = random!();
    let mut z: f32 = random!();
    x -= 0.5;
    z -= 0.5;
    x *= 2.0;
    z *= 2.0;
    let mut thing = Vec2::new(x, z);
    let (x_neg, y_neg): (f32, f32) = (random!(), random!());
    let x_neg = if x_neg < 0.5 { -0.01 } else { 0.01 };
    let y_neg = if y_neg < 0.5 { -0.01 } else { 0.01 };
    let val: f32 = random!();
    while thing.distance(Vec2::default()) < 7.0 {
        if val < 0.5 {
            thing.x += x_neg;
        } else {
            thing.y += y_neg;
        }
    }
    (x, z) = (thing.x, thing.y);
    let b: f32 = random!();
    commands.spawn(EnemyBundle {
        enemy: Enemy,
        speed: Speed(0.05),
        pbr_bundle: PbrBundle {
            mesh: meshes.add(shape::Cube::new(0.1).into()),
            material: materials.add(Color::rgb(0.3, 0.3, b).into()),
            transform: Transform::from_xyz(x, 0.1, z),
            ..default()
        },
        rigid_body: RigidBody::Dynamic,
        angular_velocity: Default::default(),
        collider: Collider::cuboid(0.1, 0.1, 0.1),
        friction: Friction::new(0.00),
    });
}

#[derive(Resource)]
struct MousePos(pub Vec3);

fn set_mouse_pos(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    pointer_location: Query<&PointerLocation>,
    picking_cameras: Query<(&Camera, &Projection, &GlobalTransform)>,
    mut mouse_pos: ResMut<MousePos>,
) {
    for pointer_loc in pointer_location.iter() {
        for (camera, proj, transform) in picking_cameras.iter() {
            let mut viewport_pos = pointer_loc.location().unwrap().position;
            //viewport_pos.y -= primary_window.single().height();
            match proj {
                Projection::Perspective(_) => {}
                Projection::Orthographic(a) => {
                    /*viewport_pos -= a.viewport_origin /*/ primary_window.single().scale_factor() as f32*/;*/
                }
            }
            viewport_pos.y = (viewport_pos.y - primary_window.single().height()) * 3.0;
            let ray = camera
                .viewport_to_world(transform, viewport_pos)
                .map(Ray3d::from)
                .unwrap();
            mouse_pos.0 = ray.origin();
        }
    }
}

fn move_enemy_to_mouse(mut commands: Commands, mouse_pos: Res<MousePos>, mut enemies: Query<(Entity, &Transform), With<Enemy>>) {
    for (enemy, enemy_pos) in enemies.iter_mut() {
        let mut direction = mouse_pos.0 - enemy_pos.translation;
        direction.y = 0.0;
        direction = direction.normalize_or_zero();
        commands.entity(enemy).insert(ExternalImpulse::new(direction.mul(0.0003)));
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 3.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // plane
    commands.spawn((PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(20.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    }, RigidBody::Static, Collider::cuboid(20.0, 0.01, 20.0)));
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });
}

use crate::main_game::mouse::MousePos;
use crate::main_game::tower::TimeSinceGameStart;
use crate::main_game::{Health, Score, Speed};
use crate::GameState;
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use random_number::random;
use std::ops::Mul;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_enemy_to_mouse, set_color_to_health, spawn_enemies)
                .run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.add_systems(
            PostUpdate,
            if_health_below_zero_then_die.run_if(state_exists_and_equals(GameState::InGame)),
        );
    }
}

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
    pub health: Health,
}

fn move_enemy_to_mouse(
    mut commands: Commands,
    mouse_pos: Res<MousePos>,
    mut enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (enemy, enemy_pos) in enemies.iter_mut() {
        let mut direction = mouse_pos.0 - enemy_pos.translation;
        direction.y = 0.0;
        direction = direction.normalize_or_zero();
        commands
            .entity(enemy)
            .try_insert(ExternalImpulse::new(direction.mul(0.0003)));
    }
}

fn set_color_to_health(
    enemies: Query<(&Health, &Handle<StandardMaterial>), With<Enemy>>,
    mut standard_material: ResMut<Assets<StandardMaterial>>,
) {
    for (health, handle) in enemies.iter() {
        standard_material
            .get_mut(handle)
            .unwrap()
            .base_color
            .set_b(health.0);
    }
}

fn if_health_below_zero_then_die(
    mut enemies: Query<(Entity, &Transform, &Health), With<Enemy>>,
    mut commands: Commands,
    mut score: ResMut<Score>,
    asset_server: Res<AssetServer>,
) {
    for (enemy, transform, health) in enemies.iter_mut() {
        if health.0 <= 0.0 {
            score.0 += 1;
            commands.entity(enemy).despawn();
            commands.spawn((
                AudioBundle {
                    source: asset_server.load("enemy_death.ogg"),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new_absolute(0.1),
                        speed: 2.0,
                        paused: false,
                        spatial: false,
                    },
                },
                transform.clone(),
                GlobalTransform::default(),
            ));
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    time_since_game_start: Res<TimeSinceGameStart>,
) {
    let val: f32 = random!();
    let chance = (time_since_game_start.0 + 2.0) / 1000.0;
    println!("chance: {}", chance);
    if val > chance {
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
    commands.spawn(EnemyBundle {
        enemy: Enemy,
        speed: Speed(0.05),
        pbr_bundle: PbrBundle {
            mesh: meshes.add(shape::Cube::new(0.1).into()),
            material: materials.add(Color::rgb(0.3, 0.3, 1.0).into()),
            transform: Transform::from_xyz(x, 0.2, z),
            ..default()
        },
        rigid_body: RigidBody::Dynamic,
        angular_velocity: Default::default(),
        collider: Collider::cuboid(0.1, 0.1, 0.1),
        friction: Friction::new(0.00),
        health: Health(1.0),
    });
}

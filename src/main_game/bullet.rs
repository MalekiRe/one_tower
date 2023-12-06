use crate::main_game::enemy::Enemy;
use crate::main_game::tower::{Tower, TowerLevel};
use crate::main_game::Health;
use crate::{AttackRadiusLvl, DamageLvl, GameState, GameStateChange};
use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use leafwing_input_manager::orientation::Orientation;
use random_number::random;
use std::ops::Mul;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_bullet.run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.add_systems(Update, fly_to_enemy);
        app.add_systems(Update, destroy_enemy);
        app.add_systems(Update, despawn_if_game_changed);
    }
}

#[derive(Component)]
pub struct Bullet {
    target: Entity,
    damage: f32,
}

#[derive(Bundle)]
pub struct BulletBundle {
    bullet: Bullet,
    pbr_bundle: PbrBundle,
}

fn spawn_bullet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut towers: Query<(&Transform, &mut Tower, &TowerLevel), Without<Enemy>>,
    asset_server: Res<AssetServer>,
    attack_radius_lvl: Res<AttackRadiusLvl>,
    damage_lvl: Res<DamageLvl>,
) {
    let attack_radius = attack_radius_lvl.0 as f32 / 15.0 + 1.0;
    let damage = damage_lvl.0 as f32 / 30.0 + 0.2;
    let mut number_of_shots = 0;
    for (tower_pos, mut tower, tower_level) in towers.iter_mut() {
        tower.0.tick(time.delta());
        if tower.0.finished() {
            for (e, transform) in enemies.iter() {
                let mut shoot = false;
                if tower_pos.translation.distance(transform.translation) <= attack_radius {
                    shoot = true;
                    commands.spawn(BulletBundle {
                        bullet: Bullet { target: e, damage },
                        pbr_bundle: PbrBundle {
                            mesh: meshes.add(
                                shape::Icosphere {
                                    radius: damage / 15.0,
                                    subdivisions: 10,
                                }
                                .try_into()
                                .unwrap(),
                            ),
                            material: materials
                                .add(StandardMaterial::from(Color::rgb(1.0, 0.6, 0.1))),
                            transform: Transform::default().with_translation(tower_pos.translation),
                            ..default()
                        },
                    });
                }
                if shoot {
                    number_of_shots += 1;
                    if number_of_shots >= 6 {
                        continue;
                    }
                    commands.spawn(
                        (AudioBundle {
                            source: asset_server.load("shooting.ogg"),
                            settings: PlaybackSettings {
                                mode: PlaybackMode::Despawn,
                                volume: Volume::new_relative(0.03),
                                speed: 1.3,
                                paused: false,
                                spatial: false,
                            },
                        }),
                    );
                }
            }
        }
    }
}

fn fly_to_enemy(
    mut commands: Commands,
    mut bullets: Query<(Entity, &Bullet, &mut Transform), Without<Enemy>>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for (entity, bullet, mut bullet_pos) in bullets.iter_mut() {
        if let Ok(target_enemy) = enemies.get(bullet.target) {
            let mut direction = target_enemy.translation - bullet_pos.translation;
            direction = direction.normalize_or_zero();
            bullet_pos.translation += direction.mul(Vec3::splat(0.1));
        } else {
            commands.entity(entity).despawn();
        }
    }
}

fn destroy_enemy(
    mut commands: Commands,
    bullets: Query<(Entity, &Bullet, &Transform), Without<Enemy>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
    asset_server: Res<AssetServer>,
) {
    const DESTROY_DISTANCE: f32 = 0.05;
    for (bullet_entity, bullet, bullet_pos) in bullets.iter() {
        if let Ok((target_transform, mut health)) = enemies.get_mut(bullet.target) {
            if target_transform
                .translation
                .distance(bullet_pos.translation)
                <= DESTROY_DISTANCE
            {
                health.0 -= bullet.damage;
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}

fn despawn_if_game_changed(
    mut commands: Commands,
    mut event_reader: EventReader<GameStateChange>,
    query: Query<Entity, With<Bullet>>,
) {
    for ev in event_reader.read() {
        match ev {
            GameStateChange::Staging => {}
            GameStateChange::MainGame => {
                for q in query.iter() {
                    commands.entity(q).despawn();
                }
                break;
            }
        }
    }
    event_reader.clear();
}

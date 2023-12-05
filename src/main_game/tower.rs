use crate::main_game::mouse::MousePos;
use crate::main_game::{calculate_available_towers, PlacedTowers, Score};
use crate::{GameState, GameStateChange, UpgradeRadiusLvl};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use random_number::random;
use std::time::Duration;

pub struct TowerPlugin;
impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_tower, set_tower_size)
                .chain()
                .run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.add_systems(
            Update,
            tower_progress_increase.run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.add_systems(Update, clone_material);
        app.add_systems(Update, set_tower_duration);
        app.add_systems(PostUpdate, on_game_end);
        app.insert_resource(TimeSinceGameStart(0.0));
    }
}

fn on_game_end(
    query: Query<Entity, With<Tower>>,
    mut event_reader: EventReader<GameStateChange>,
    mut commands: Commands,
    mut time_since_game_start: ResMut<TimeSinceGameStart>,
) {
    for e in event_reader.read() {
        match e {
            GameStateChange::Staging => {
                for e in query.iter() {
                    commands.entity(e).despawn_recursive();
                }
                break;
            }
            GameStateChange::MainGame => {
                time_since_game_start.0 = 0.0;
            }
        }
    }
    event_reader.clear();
}

#[derive(Component)]
pub struct Tower(pub Timer);
#[derive(Component)]
pub struct TowerLevel(u32);

#[derive(Component)]
struct TowerProgress(f32);

fn set_tower_size(mut query: Query<(&mut Transform, &TowerLevel), With<Tower>>) {
    for (mut transform, tower_level) in query.iter_mut() {
        transform.scale = Vec3::splat(((tower_level.0 as f32) / 125.0) + 0.1)
    }
}

fn set_tower_duration(mut query: Query<(&mut Tower, &TowerLevel), Changed<TowerLevel>>) {
    for (mut tower, tower_level) in query.iter_mut() {
        tower.0.set_duration(Duration::from_millis(
            (1000.0 / ((tower_level.0 as f32).log(1.5) + 1.0)) as u64,
        ))
    }
}

#[derive(Resource)]
pub struct TimeSinceGameStart(pub(crate) f32);

fn spawn_tower(
    mouse_pos: Res<MousePos>,
    mut mouse_event_reader: EventReader<MouseButtonInput>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    score: Res<Score>,
    mut placed: ResMut<PlacedTowers>,
    mut event_reader: EventReader<GameStateChange>,
    towers: Query<&Transform, With<Tower>>,
    time: Res<Time>,
    mut last_elapsed: Local<f32>,
    mut time_since_game_start: ResMut<TimeSinceGameStart>,
) {
    time_since_game_start.0 += time.delta_seconds();
    if !event_reader.is_empty()
        || time_since_game_start.0 < 1.0
        || *last_elapsed + 0.4 > time.elapsed_seconds()
    {
        event_reader.clear();
        mouse_event_reader.clear();
        return;
    }
    for event in mouse_event_reader.read() {
        match event.button {
            MouseButton::Left => {
                if calculate_available_towers(*score, *placed) <= 0 {
                    return;
                }
                placed.0 += 1;

                *last_elapsed = time.elapsed_seconds();

                let mut this_pos = Vec3::new(mouse_pos.0.x, 0.3, mouse_pos.0.z);
                let mut intersects = true;
                let move_x: f32 = random!();
                while intersects {
                    //println!("intersects");
                    intersects = false;
                    for tower in towers.iter() {
                        if tower.translation.distance(this_pos) <= 0.7 {
                            intersects = true;
                        }
                    }
                    if intersects {
                        if move_x <= 0.25 {
                            this_pos.x += 0.01;
                            this_pos.z += 0.01;
                        } else if move_x <= 0.5 {
                            this_pos.x += 0.01;
                            this_pos.z -= 0.01;
                        } else if move_x < 0.75 {
                            this_pos.x -= 0.01;
                            this_pos.z += 0.01;
                        } else {
                            this_pos.x -= 0.01;
                            this_pos.z -= 0.01;
                        }
                    }
                }

                commands.spawn((
                    SceneBundle {
                        scene: asset_server.load("tower.glb#Scene0"),
                        transform: Transform::default()
                            .with_scale(Vec3::splat(0.1))
                            .with_translation(this_pos),
                        ..default()
                    },
                    Tower(Timer::new(
                        Duration::from_millis(1000),
                        TimerMode::Repeating,
                    )),
                    TowerLevel(1),
                    TowerProgress(0.0),
                ));
                mouse_event_reader.clear();
                return;
            }
            _ => {}
        }
    }
}

fn clone_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    tower_query: Query<Entity, With<Tower>>,
    children_query: Query<&Children>,
    mut query: Query<&mut Handle<StandardMaterial>, Added<Handle<StandardMaterial>>>,
) {
    for tower in tower_query.iter() {
        for child in children_query.iter_descendants(tower) {
            if let Ok(mut mat) = query.get_mut(child) {
                let material2 = materials.get(&*mat).unwrap().clone();
                *mat = materials.add(material2);
            }
        }
    }
}

fn tower_progress_increase(
    mouse_pos: Res<MousePos>,
    mouse_button: Res<Input<MouseButton>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tower_query: Query<
        (
            Entity,
            &Transform,
            &mut TowerProgress,
            &mut TowerLevel,
            &Tower,
        ),
        With<Tower>,
    >,
    children_query: Query<&Children>,
    children_material_query: Query<&Handle<StandardMaterial>>,
    time: Res<Time>,
    upgrade_radius_lvl: Res<UpgradeRadiusLvl>,
) {
    let upgrade_radius = (upgrade_radius_lvl.0 as f32).log(1.1) / 25.0 + 0.5;

    for (tower_entity, tower_pos, mut tower_progress, mut tower_level, tower) in
        tower_query.iter_mut()
    {
        let mut t_pos = tower_pos.translation;
        t_pos.y = 0.0;
        let mut m_pos = mouse_pos.0;
        m_pos.y = 0.0;
        if t_pos.distance(m_pos) <= upgrade_radius {
            tower_progress.0 += 1.0 / (20.0 * (tower_level.0 as f32 + 5.0));

            for mat in children_query.iter_descendants(tower_entity) {
                if let Ok(mat) = children_material_query.get(mat) {
                    materials
                        .get_mut(mat)
                        .unwrap()
                        .base_color
                        .set_b(0.5)
                        .set_g(0.5);
                }
            }
        } else {
            for mat in children_query.iter_descendants(tower_entity) {
                if let Ok(mat) = children_material_query.get(mat) {
                    materials
                        .get_mut(mat)
                        .unwrap()
                        .base_color
                        .set_b(0.3)
                        .set_g(0.3);
                }
            }
        }

        let mut index = 0;
        for mat in children_query.iter_descendants(tower_entity) {
            if let Ok(mat) = children_material_query.get(mat) {
                index += 1;
                if index == 6 {
                    let left = tower.0.percent_left();
                    materials
                        .get_mut(mat)
                        .unwrap()
                        .base_color
                        .set_r((left).max(0.5));
                }
            }
        }

        if tower_progress.0 >= 1.0 {
            tower_progress.0 = 0.0;
            tower_level.0 += 1;
        }
    }
}

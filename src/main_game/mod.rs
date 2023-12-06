mod bullet;
mod enemy;
mod mouse;
mod tower;

use crate::main_game::bullet::BulletPlugin;
use crate::main_game::enemy::{Enemy, EnemyPlugin};
use crate::main_game::mouse::MousePlugin;
use crate::main_game::tower::{TimeSinceGameStart, TowerPlugin};
use crate::{GameState, GameStateChange, Gold, GoldConversionRateLvl};
use bevy::app::{App, PluginGroupBuilder};
use bevy::prelude::*;
use bevy_egui::EguiContexts;

pub struct MainGamePlugin;

pub struct MainGamePlugins;

impl PluginGroup for MainGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<MainGamePlugins>()
            .add(MainGamePlugin)
            .add(TowerPlugin)
            .add(MousePlugin)
            .add(EnemyPlugin)
            .add(BulletPlugin)
    }
}

impl Plugin for MainGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            on_die.run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.insert_resource(Score(0));
        app.insert_resource(PlacedTowers(0));
        app.add_systems(
            Update,
            ui.run_if(state_exists_and_equals(GameState::InGame)),
        );
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut event_writer: EventWriter<GameStateChange>,
    score: ResMut<Score>,
    placed_towers: Res<PlacedTowers>,
    time_since_game_start: Res<TimeSinceGameStart>,
) {
    let ctx = contexts.ctx_mut();
    egui::SidePanel::left("my_left")
        .resizable(false)
        .show(ctx, |ui| {
            if ui.button("end game").clicked() {
                event_writer.send(GameStateChange::Staging);
            }
            ui.label(format!("score: {}", score.0));
            ui.label(format!(
                "avaliable towers: {}",
                calculate_available_towers(*score, *placed_towers)
            ));
            ui.label(format!("minutes elapsed: {}", time_since_game_start.0 / 60.0));
            ui.label(format!("time goal: {}", 6.666))
        });
}

#[derive(Resource, Clone, Copy)]
pub struct Score(pub u32);

#[derive(Resource, Clone, Copy)]
pub struct PlacedTowers(pub u32);

pub fn calculate_available_towers(score: Score, placed_towers: PlacedTowers) -> u32 {
    (1 + (score.0 / 50)) - placed_towers.0
}

#[derive(Component)]
pub struct Speed(f32);

fn on_die(
    mut event_reader: EventReader<GameStateChange>,
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    mut score: ResMut<Score>,
    mut placed_towers: ResMut<PlacedTowers>,
    mut gold: ResMut<Gold>,
    gold_conversion_rate_lvl: Res<GoldConversionRateLvl>,
) {
    for ev in event_reader.read() {
        match ev {
            GameStateChange::Staging => {
                for enemy in enemies.iter() {
                    commands.entity(enemy).despawn();
                }
                gold.0 += (score.0 as f32) / ((gold_conversion_rate_lvl.0 as f32).log(1.5) / 5.0);
                score.0 = 0;
                placed_towers.0 = 0;
            }
            GameStateChange::MainGame => {}
        }
    }
}

#[derive(Component)]
pub struct Health(f32);

mod main_game;

use crate::main_game::MainGamePlugins;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy::asset::AssetMetaCheck;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_mod_picking::debug::DebugPickingPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_xpbd_3d::plugins::PhysicsPlugins;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};
use egui::CollapsingHeader;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins(DefaultPlugins)
        .add_plugins(
            DefaultPickingPlugins
                .build()
                .disable::<DebugPickingPlugin>(),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(EguiPlugin)
        .add_plugins(MainGamePlugins)
        .add_state::<GameState>()
        .add_event::<GameStateChange>()
        .add_systems(Startup, setup)
        .add_systems(Update, change_game_state)
        .add_plugins(StagingPlugin)
        .run();
}

pub struct StagingPlugin;

impl Plugin for StagingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_game_ui.run_if(state_exists_and_equals(GameState::Staging)),
        );
        app.insert_resource(Gold(0.0));
        app.insert_resource(UpgradeRadiusLvl(1));
        app.insert_resource(DamageLvl(1));
        app.insert_resource(AttackRadiusLvl(1));
        app.insert_resource(GoldConversionRateLvl(1));
    }
}

fn start_game_ui(
    mut gold: ResMut<Gold>,
    mut upgrade_radius_lvl: ResMut<UpgradeRadiusLvl>,
    mut attack_radius_lvl: ResMut<AttackRadiusLvl>,
    mut damage_lvl: ResMut<DamageLvl>,
    mut gold_conversion_rate_lvl: ResMut<GoldConversionRateLvl>,
    mut contexts: EguiContexts,
    mut event_writer: EventWriter<GameStateChange>,
) {
    let ctx = contexts.ctx_mut();
    egui::SidePanel::left("left")
        .resizable(false)
        .show(ctx, |ui| {
            if ui.button("start game").clicked() {
                event_writer.send(GameStateChange::MainGame)
            }
            CollapsingHeader::new("upgrades")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("gold {}", gold.0));
                    if calculate_cost_to_upgrade(upgrade_radius_lvl.0) <= gold.0 as u32 {
                        if ui
                            .button(format!("upgrade radius level: {}", upgrade_radius_lvl.0))
                            .clicked()
                        {
                            gold.0 -= calculate_cost_to_upgrade(upgrade_radius_lvl.0) as f32;
                            upgrade_radius_lvl.0 += 1;
                        }
                    } else {
                        ui.label(format!("upgrade radius level: {}", upgrade_radius_lvl.0));
                        ui.label(format!(
                            "needed to upgrade: {}",
                            calculate_cost_to_upgrade(upgrade_radius_lvl.0)
                        ));
                    }
                    if calculate_cost_to_upgrade(attack_radius_lvl.0) <= gold.0 as u32 {
                        if ui
                            .button(format!("attack radius level: {}", attack_radius_lvl.0))
                            .clicked()
                        {
                            gold.0 -= calculate_cost_to_upgrade(attack_radius_lvl.0) as f32;
                            attack_radius_lvl.0 += 1;
                        }
                    } else {
                        ui.label(format!("attack radius level: {}", attack_radius_lvl.0));
                        ui.label(format!(
                            "needed to upgrade: {}",
                            calculate_cost_to_upgrade(attack_radius_lvl.0)
                        ));
                    }
                    if calculate_cost_to_upgrade(damage_lvl.0) <= gold.0 as u32 {
                        if ui
                            .button(format!("damage level: {}", damage_lvl.0))
                            .clicked()
                        {
                            gold.0 -= calculate_cost_to_upgrade(damage_lvl.0) as f32;
                            damage_lvl.0 += 1;
                        }
                    } else {
                        ui.label(format!("damage level: {}", damage_lvl.0));
                        ui.label(format!(
                            "needed to upgrade: {}",
                            calculate_cost_to_upgrade(damage_lvl.0)
                        ));
                    }
                    if calculate_cost_to_upgrade(gold_conversion_rate_lvl.0) <= gold.0 as u32 {
                        if ui
                            .button(format!(
                                "gold conversion rate level: {}",
                                gold_conversion_rate_lvl.0
                            ))
                            .clicked()
                        {
                            gold.0 -= calculate_cost_to_upgrade(gold_conversion_rate_lvl.0) as f32;
                            gold_conversion_rate_lvl.0 += 1;
                        }
                    } else {
                        ui.label(format!(
                            "gold conversion rate level: {}",
                            gold_conversion_rate_lvl.0
                        ));
                        ui.label(format!(
                            "needed to upgrade: {}",
                            calculate_cost_to_upgrade(gold_conversion_rate_lvl.0)
                        ));
                    }
                });
            CollapsingHeader::new("stats")
                .default_open(true)
                .show(ui, |ui| {
                    let attack_radius = attack_radius_lvl.0 as f32 / 35.0 + 1.0;
                    let damage = damage_lvl.0 as f32 / 30.0 + 0.2;
                    let gold_conversion_rate =
                        1.0 / (5.0 - (gold_conversion_rate_lvl.0 as f32).log(2.5));
                    let upgrade_radius = (upgrade_radius_lvl.0 as f32).log(1.1) / 25.0 + 0.5;
                    ui.label(format!("attack radius: {}", attack_radius));
                    ui.label(format!("bullet damage: {}", damage));
                    ui.label(format!("gold conversion rate: {}", gold_conversion_rate));
                    ui.label(format!("upgrade radius: {}", upgrade_radius));
                });
        });
}

fn change_game_state(
    mut event_reader: EventReader<GameStateChange>,
    mut state: ResMut<NextState<GameState>>,
) {
    for event in event_reader.read() {
        match event {
            GameStateChange::Staging => state.set(GameState::Staging),
            GameStateChange::MainGame => state.set(GameState::InGame),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum GameState {
    #[default]
    Staging,
    InGame,
}

#[derive(Event)]
pub enum GameStateChange {
    Staging,
    MainGame,
}

#[derive(Resource)]
pub struct Gold(f32);

#[derive(Resource)]
pub struct UpgradeRadiusLvl(u32);

#[derive(Resource)]
pub struct AttackRadiusLvl(u32);

#[derive(Resource)]
pub struct DamageLvl(u32);

#[derive(Resource)]
pub struct GoldConversionRateLvl(u32);

pub fn calculate_cost_to_upgrade(level: u32) -> u32 {
    level + (2 * level.ilog2())
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
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(20.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(20.0, 0.01, 20.0),
    ));
    // light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..default()
    });
}

use crate::main_game::enemy::Enemy;
use crate::{GameState, GameStateChange};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_picking::backends::raycast::bevy_mod_raycast::prelude::Ray3d;
use bevy_mod_picking::prelude::PointerLocation;

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos::default());
        app.add_systems(
            Update,
            set_mouse_pos.run_if(state_exists_and_equals(GameState::InGame)),
        );
        app.add_systems(
            Update,
            kill_player.run_if(state_exists_and_equals(GameState::InGame)),
        );
    }
}

#[derive(Resource, Default)]
pub struct MousePos(pub Vec3);

fn set_mouse_pos(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    pointer_location: Query<&PointerLocation>,
    picking_cameras: Query<(&Camera, &GlobalTransform)>,
    mut mouse_pos: ResMut<MousePos>,
) {
    for pointer_loc in pointer_location.iter() {
        for (camera, transform) in picking_cameras.iter() {
            let mut viewport_pos = pointer_loc.location().unwrap().position;
            viewport_pos.y = (viewport_pos.y - primary_window.single().height()) * 3.0;
            let ray = camera
                .viewport_to_world(transform, viewport_pos)
                .map(Ray3d::from)
                .unwrap();
            mouse_pos.0 = ray.origin();
        }
    }
}

fn kill_player(
    enemies: Query<&Transform, With<Enemy>>,
    mouse: ResMut<MousePos>,
    mut event_writer: EventWriter<GameStateChange>,
) {
    let mut mouse = mouse.0;
    mouse.y = 0.0;
    for enemy in enemies.iter() {
        let mut t = enemy.translation;
        t.y = 0.0;
        if mouse.distance(t) < 0.03 {
            println!("dead");
            event_writer.send(GameStateChange::Staging)
        }
    }
}

use bevy::prelude::*;

mod components;
mod resources;
mod systems;

pub use resources::{Kcl, KclFlag, KCL_COLOURS};
pub struct KclPlugin;

impl Plugin for KclPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(systems::spawn_model)
            .add_system(systems::update_kcl_model);
    }
}
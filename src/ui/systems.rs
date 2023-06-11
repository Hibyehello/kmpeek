use super::resources::AppState;
use crate::{
    camera::{
        CameraMode, CameraSettings, FlyCam, FlySettings, OrbitCam, OrbitSettings, TopDownCam,
        TopDownSettings,
    },
    kcl::*,
    kmp::*,
};
use bevy::{math::vec3, prelude::*};
use bevy_egui::{egui, EguiContexts};

pub fn update_ui(
    mut contexts: EguiContexts,
    mut kcl: ResMut<Kcl>,
    mut app_state: ResMut<AppState>,
    mut camera_settings: ResMut<CameraSettings>,
    mut fly_cam_transform: Query<
        &mut Transform,
        (With<FlyCam>, Without<OrbitCam>, Without<TopDownCam>),
    >,
    mut orbit_cam_transform: Query<
        &mut Transform,
        (Without<FlyCam>, With<OrbitCam>, Without<TopDownCam>),
    >,
    mut topdown_cam: Query<
        (&mut Transform, &mut Projection),
        (Without<FlyCam>, Without<OrbitCam>, With<TopDownCam>),
    >,
) {
    let ctx = contexts.ctx_mut();
    let mut fly_cam_transform = fly_cam_transform
        .get_single_mut()
        .expect("Could not get single fly cam");
    let mut orbit_cam_transform = orbit_cam_transform
        .get_single_mut()
        .expect("Could not get single orbit cam");
    let (mut topdown_cam_transform, mut topdown_cam_projection) = topdown_cam
        .get_single_mut()
        .expect("Could not get single topdown cam");

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    // …
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    // …
                }
                if ui.button("Redo").clicked() {
                    // …
                }
            });
        });
    });

    let mut customise_kcl_open = app_state.customise_kcl_open;
    egui::Window::new("Customise Collision Model")
        .open(&mut customise_kcl_open)
        .collapsible(false)
        .min_width(300.)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Check All").clicked() {
                        for vertex_group in kcl.vertex_groups.iter_mut() {
                            vertex_group.visible = true;
                        }
                    }
                    if ui.button("Uncheck All").clicked() {
                        for vertex_group in kcl.vertex_groups.iter_mut() {
                            vertex_group.visible = false;
                        }
                    }
                    if ui.button("Reset").clicked() {
                        for (i, vertex_group) in kcl.vertex_groups.iter_mut().enumerate() {
                            vertex_group.visible = true;
                            vertex_group.colour = KCL_COLOURS[i];
                        }
                    }
                });
                ui.separator();
                // this macro means that the same ui options can be repeated without copy and pasting it 32 times
                macro_rules! kcl_type_options {
                    ($name:expr, $i:expr) => {
                        ui.horizontal(|ui| {
                            let (mut colour, mut visible) =
                                (kcl.vertex_groups[$i].colour, kcl.vertex_groups[$i].visible);
                            ui.color_edit_button_rgba_unmultiplied(&mut colour);
                            ui.checkbox(&mut visible, $name);
                            // only update the kcl if the variables have been changed in the UI
                            if colour != kcl.vertex_groups[$i].colour
                                || visible != kcl.vertex_groups[$i].visible
                            {
                                kcl.vertex_groups[$i].colour = colour;
                                kcl.vertex_groups[$i].visible = visible;
                            }
                        });
                        ui.separator();
                    };
                }
                kcl_type_options!("Road1", 0);
                kcl_type_options!("SlipperyRoad1", 1);
                kcl_type_options!("WeakOffroad", 2);
                kcl_type_options!("Offroad", 3);
                kcl_type_options!("HeavyOffroad", 4);
                kcl_type_options!("SlipperyRoad2", 5);
                kcl_type_options!("BoostPanel", 6);
                kcl_type_options!("BoostRamp", 7);
                kcl_type_options!("SlowRamp", 8);
                kcl_type_options!("ItemRoad", 9);
                kcl_type_options!("SolidFall", 10);
                kcl_type_options!("MovingWater", 11);
                kcl_type_options!("Wall1", 12);
                kcl_type_options!("InvisibleWall1", 13);
                kcl_type_options!("ItemWall", 14);
                kcl_type_options!("Wall2", 15);
                kcl_type_options!("FallBoundary", 16);
                kcl_type_options!("CannonTrigger", 17);
                kcl_type_options!("ForceRecalculation", 18);
                kcl_type_options!("HalfPipeRamp", 19);
                kcl_type_options!("PlayerOnlyWall", 20);
                kcl_type_options!("MovingRoad", 21);
                kcl_type_options!("StickyRoad", 22);
                kcl_type_options!("Road2", 23);
                kcl_type_options!("SoundTrigger", 24);
                kcl_type_options!("WeakWall", 25);
                kcl_type_options!("EffectTrigger", 26);
                kcl_type_options!("ItemStateModifier", 27);
                kcl_type_options!("HalfPipeInvisibleWall", 28);
                kcl_type_options!("RotatingRoad", 29);
                kcl_type_options!("SpecialWall", 30);
                kcl_type_options!("InvisibleWall2", 31);
            });
        });
    if customise_kcl_open != app_state.customise_kcl_open {
        app_state.customise_kcl_open = customise_kcl_open;
    }

    let mut camera_settings_open = app_state.camera_settings_open;
    egui::Window::new("Camera Settings")
        .open(&mut camera_settings_open)
        .collapsible(false)
        .min_width(300.)
        .show(ctx, |ui| {
            if ui.button("Reset Positions").clicked() {
                *fly_cam_transform = Transform::from_translation(FlySettings::default().start_pos)
                    .looking_at(Vec3::ZERO, Vec3::Y);
                *orbit_cam_transform =
                    Transform::from_translation(OrbitSettings::default().start_pos)
                        .looking_at(Vec3::ZERO, Vec3::Y);
                *topdown_cam_transform = Transform::from_translation(vec3(
                    TopDownSettings::default().start_pos.x,
                    TopDownSettings::default().y_pos,
                    TopDownSettings::default().start_pos.y,
                ))
                .looking_at(Vec3::ZERO, Vec3::Z);
                *topdown_cam_projection = Projection::Orthographic(OrthographicProjection {
                    near: 0.00001,
                    far: 100000.,
                    scale: 100.,
                    ..default()
                });
            }
            if ui.button("Reset Settings").clicked() {
                *camera_settings = CameraSettings::default();
            }
            ui.collapsing("Fly Camera", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Look Sensitivity")
                        .on_hover_text("How sensitive the camera rotation is to mouse movements");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.fly.look_sensitivity).speed(0.1),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Speed").on_hover_text("How fast the camera moves");
                    ui.add(egui::DragValue::new(&mut camera_settings.fly.speed).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Speed Boost").on_hover_text(
                        "How much faster the camera moves when holding the speed boost button",
                    );
                    ui.add(egui::DragValue::new(&mut camera_settings.fly.speed_boost).speed(0.1));
                });
            });
            ui.collapsing("Orbit Camera", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Rotate Sensitivity")
                        .on_hover_text("How sensitive the camera rotation is to mouse movements");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.orbit.rotate_sensitivity)
                            .speed(0.1),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Pan Sensitivity:")
                        .on_hover_text("How sensitive the camera panning is to mouse movements");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.orbit.pan_sensitivity).speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Scroll Sensitivity")
                        .on_hover_text("How sensitive the camera zoom is to scrolling");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.orbit.scroll_sensitivity)
                            .speed(0.1),
                    );
                });
            });
            ui.collapsing("Top Down Camera", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Move Sensitivity")
                        .on_hover_text("How sensitive the camera movement is to mouse movements");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.top_down.move_sensitivity)
                            .speed(0.1),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Scroll Sensitivity")
                        .on_hover_text("How sensitive the camera zoom is to scrolling");
                    ui.add(
                        egui::DragValue::new(&mut camera_settings.top_down.scroll_sensitivity)
                            .speed(0.1),
                    );
                });
            });
        });
    if camera_settings_open != app_state.camera_settings_open {
        app_state.camera_settings_open = camera_settings_open;
    }

    egui::SidePanel::left("side_panel")
        .resizable(true)
        .show(ctx, |ui| {
            egui::CollapsingHeader::new("View Options")
                .default_open(true)
                .show_background(true)
                .show(ui, |ui| {
                    ui.collapsing("Collision Model", |ui| {
                        let (
                            mut show_walls,
                            mut show_invisible_walls,
                            mut show_death_barriers,
                            mut show_effects_triggers,
                        ) = (
                            app_state.show_walls,
                            app_state.show_invisible_walls,
                            app_state.show_death_barriers,
                            app_state.show_effects_triggers,
                        );
                        ui.checkbox(&mut show_walls, "Show Walls");
                        ui.checkbox(&mut show_invisible_walls, "Show Invisible Walls");
                        ui.checkbox(&mut show_death_barriers, "Show Death Barriers");
                        ui.checkbox(&mut show_effects_triggers, "Show Effects & Triggers");
                        if show_walls != app_state.show_walls {
                            app_state.show_walls = show_walls;
                            kcl.vertex_groups[KclFlag::Wall1 as usize].visible = show_walls;
                            kcl.vertex_groups[KclFlag::Wall2 as usize].visible = show_walls;
                            kcl.vertex_groups[KclFlag::WeakWall as usize].visible = show_walls;
                        }
                        if show_invisible_walls != app_state.show_invisible_walls {
                            app_state.show_invisible_walls = show_invisible_walls;
                            kcl.vertex_groups[KclFlag::InvisibleWall1 as usize].visible =
                                show_invisible_walls;
                            kcl.vertex_groups[KclFlag::InvisibleWall2 as usize].visible =
                                show_invisible_walls;
                        }
                        if show_death_barriers != app_state.show_death_barriers {
                            app_state.show_death_barriers = show_death_barriers;
                            kcl.vertex_groups[KclFlag::SolidFall as usize].visible =
                                show_death_barriers;
                            kcl.vertex_groups[KclFlag::FallBoundary as usize].visible =
                                show_death_barriers;
                        }
                        if show_effects_triggers != app_state.show_effects_triggers {
                            app_state.show_effects_triggers = show_effects_triggers;
                            kcl.vertex_groups[KclFlag::ItemStateModifier as usize].visible =
                                show_effects_triggers;
                            kcl.vertex_groups[KclFlag::EffectTrigger as usize].visible =
                                show_effects_triggers;
                            kcl.vertex_groups[KclFlag::SoundTrigger as usize].visible =
                                show_effects_triggers;
                            kcl.vertex_groups[KclFlag::CannonTrigger as usize].visible =
                                show_effects_triggers;
                        }
                        if ui.button("Customise...").clicked() {
                            app_state.customise_kcl_open = true;
                        }
                    });

                    ui.collapsing("Camera", |ui| {
                        ui.horizontal(|ui| {
                            let mut mode = camera_settings.mode;
                            ui.selectable_value(&mut mode, CameraMode::Fly, "Fly");
                            ui.selectable_value(&mut mode, CameraMode::Orbit, "Orbit");
                            ui.selectable_value(&mut mode, CameraMode::TopDown, "Top Down");
                            if camera_settings.mode != mode {
                                camera_settings.mode = mode;
                            }
                        });
                        if ui.button("Camera Settings...").clicked() {
                            app_state.camera_settings_open = true;
                        }
                    });
                });

            ui.separator();
        });
}
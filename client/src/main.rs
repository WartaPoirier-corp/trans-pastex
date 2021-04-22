use bevy::prelude::*;

mod map;
mod plugin;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State {
    Loading,
    Main,
}

enum EcsToPlugin {
    TestEvent(usize, i64),
    ListPlugins,
}

enum PluginToEcs {
    TestEvent(usize, i64),
    ListPlugins(Vec<plugin::Manifest>),
}

struct PluginEvents(
    std::sync::Mutex<std::sync::mpsc::Sender<EcsToPlugin>>,
    std::sync::Mutex<std::sync::mpsc::Receiver<PluginToEcs>>,
);

fn main() {
    let (plug_tx, plug_rx) = std::sync::mpsc::channel();
    let (ecs_tx, ecs_rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        use runestick::FromValue;

        let plugins = plugin::Plugins::init();
        loop {
            match plug_rx.recv().unwrap() {
                EcsToPlugin::TestEvent(id, x) => {
                    let res = plugins.0[id].vm.clone().execute(&["test"], (x,)).unwrap().complete().unwrap();
                    let res = i64::from_value(res).unwrap();
                    ecs_tx.send(PluginToEcs::TestEvent(id, res)).unwrap();
                },
                EcsToPlugin::ListPlugins => {
                    let manifests = plugins.0.iter().map(|p| p.manifest.clone()).collect();
                    ecs_tx.send(PluginToEcs::ListPlugins(manifests)).unwrap();
                },
            }
        }
    });

    App::build()
        .add_state(State::Loading)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(PluginEvents(std::sync::Mutex::new(plug_tx), std::sync::Mutex::new(ecs_rx)))
        .insert_resource(PluginUi(None))
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_system_set(SystemSet::on_enter(State::Loading)
            .with_system(map::load_assets.system())
        )
        .add_system_set(SystemSet::on_update(State::Loading)
            .with_system(map::check_assets.system())
        )
        .add_system_set(SystemSet::on_enter(State::Main)
            .with_system(setup.system())
            .with_system(map::spawn_map.system())
        )
        .add_system_set(SystemSet::on_update(State::Main)
            .with_system(bevy::input::keyboard::keyboard_input_system.system())
            .with_system(move_player.system())
            .with_system(move_camera.system())
            .with_system(plugin_window_toggle.system())
            .with_system(plugin_window.system())
            .with_system(exit.system())
        )
        .run();
}

struct Player;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = asset_server.load("Sakura-1.gltf#Material0");
    let tree = asset_server.load("Sakura-1.gltf#Mesh0/Primitive0");
    // add entities to the world

    for i in 0..10 {
        commands.spawn()
            .insert_bundle(PbrBundle {
            mesh: tree.clone(),
            material: material.clone(),
            transform: {
                let mut transform = Transform::from_translation(Vec3::new(
                    10.0 + (i as f32).cos() * (i as f32),
                    0.0,
                    ((i as f32) + 7.0) - 2.0 * (i as f32).sin()),
                );
                transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                transform
            },
            ..Default::default()
        });
    }

    // cube
    commands.spawn().insert_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.7, 0.7).into()),
        transform: Transform::from_translation(Vec3::new(5.0, 0.5, 5.0)),
        ..Default::default()
    })
    .insert(Player);
    // light
    commands.spawn().insert_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(10.0, 8.0, 10.0)),
        ..Default::default()
    });
    // camera
    commands.spawn().insert_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(10.0, 3.0, 5.0))
            .looking_at(Vec3::new(5.0, 0.5, 5.0), Vec3::Y),
        ..Default::default()
    });
}

const SPEED: f32 = 1.0;

fn move_player(time: Res<Time>, input: Res<Input<KeyCode>>, mut player: Query<&mut Transform, With<Player>>) {
    let player = &mut player.iter_mut().next().unwrap();
    for key in input.get_pressed() {
        let mov = SPEED * time.delta_seconds();
        match key {
            KeyCode::Z | KeyCode::Up => player.translation.x -= mov,
            KeyCode::S | KeyCode::Down => player.translation.x += mov,
            KeyCode::Q | KeyCode::Left => player.translation.z += mov,
            KeyCode::D | KeyCode::Right => player.translation.z -= mov,
            _ => {},
        }
    }
}

fn move_camera(mut query: QuerySet<(Query<(&mut Transform, &bevy::render::camera::Camera)>, Query<(&Transform, &Player)>)>) {
    let player_pos = {
        let player = query.q1();
        let player = player.iter().next().unwrap().0;
        player.translation
    };
    let cam = query.q0_mut();
    let cam = &mut cam.iter_mut().next().unwrap().0;
    cam.translation = player_pos + Vec3::new(5.0, 3.0, 0.0);
}

struct PluginUi(Option<Vec<plugin::Manifest>>);

fn plugin_window_toggle(input: Res<Input<KeyCode>>, plugins: Res<PluginEvents>, mut ui: ResMut<PluginUi>) {
    for key in input.get_just_released() {
        match key {
            KeyCode::P => {
                match ui.0 {
                    Some(_) => ui.0 = None,
                    None => {
                        let tx = plugins.0.lock().unwrap();
                        tx.send(EcsToPlugin::ListPlugins).ok();
                        let rx = plugins.1.lock().unwrap();
                        match rx.recv().unwrap() {
                            PluginToEcs::ListPlugins(manifests) => {
                                ui.0 = Some(manifests)
                            }
                            _ => {},
                        }
                    }
                }
            }
            _ => {},
        }
    }
}

fn plugin_window(egui: Res<bevy_egui::EguiContext>, plugins: Res<PluginUi>) {
    if let Some(ref manifests) = plugins.0 {
        bevy_egui::egui::Window::new("Plugins").show(egui.ctx(), |ui| {
            bevy_egui::egui::Grid::new("plugins_table").show(ui, |ui| {
                for man in manifests {
                    ui.label(&man.name);
                    ui.label(&man.description);
                    ui.label(format!("Par {}", man.authors[0]));
                    ui.label(format!("Inclus {} items", man.items.len()));
                    ui.end_row();
                }
            });
        });
    }
}

fn exit(input: Res<Input<KeyCode>>, mut exit_event: EventWriter<bevy::app::AppExit>) {
    for key in input.get_just_pressed() {
        if *key == KeyCode::Escape {
            exit_event.send(bevy::app::AppExit);
        }
    }
}

use bevy::prelude::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(build_map())
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(bevy::input::keyboard::keyboard_input_system.system())
        .add_system(move_player.system())
        .add_system(move_camera.system())
        .add_system(exit.system())
        .run();
}

fn build_map() -> Map {
    Map {
        dimensions: (20, 20),
        ground: {
            let mut ground = Vec::with_capacity(20 * 20);
            for i in 0..20 {
                for j in 0..20 {
                    let t = if 10 < i && i < 15 && (j - i) < 2 {
                        GroundType::Dirt
                    } else if i < 4 && j <= i {
                        GroundType::Water
                    } else if i % 7 == j {
                        GroundType::Rock
                    } else {
                        GroundType::Grass
                    };

                    let height = if t == GroundType::Water {
                        0
                    } else if (i % 3 == 0 && j % 5 == 0) || (i + j % 13 < 2) {
                        1
                    } else if i > 15 {
                        (j + i - 15) / 5
                    } else {
                        0
                    };

                    ground.push((t, height as u32));
                }
            }
            ground
        },
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GroundType {
    Grass,
    Dirt,
    Water,
    Rock,
}

struct Map {
    dimensions: (u32, u32),
    ground: Vec<(GroundType, u32)>,
}

impl Map {
    fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        let mut positions = Vec::with_capacity(self.ground.len());
        let mut normals = Vec::with_capacity(self.ground.len());
        let mut indices = Vec::with_capacity(self.ground.len() * 3 * 2);
        let mut uv = Vec::with_capacity(self.ground.len());
        let (i, j) = self.dimensions;
        for x in 0..i {
            for y in 0..j {
                let (_t, h) = self.ground[(x as usize) * 20 + (y as usize)];
                positions.push([x as f32, (h as f32) * 0.5, y as f32]);

                // TODO: this is probably very wrong
                let prev_h = if x > 0 && y < 19 {
                    let top_right_index = (x - 1) * 20 + y + 1;
                    self.ground[top_right_index as usize].1 as f32
                } else {
                    h as f32
                };
                let vec = Vec3::new(1.0, (h as f32) - prev_h, 1.0);
                let normal = vec.normalize().any_orthonormal_vector();
                normals.push([normal[0], normal[1], normal[2]]);

                uv.push([(x as f32) / (i as f32), (y as f32) / (j as f32)]);

                if x > 0 && y < 19 {
                    let top_index = (x - 1) * 20 + y;
                    let right_index = x * 20 + y + 1;
                    let top_right_index = (x - 1) * 20 + y + 1;
                    let current_index = x * 20 + y;

                    indices.push(top_index);
                    indices.push(top_right_index);
                    indices.push(current_index);

                    indices.push(current_index);
                    indices.push(top_right_index);
                    indices.push(right_index);
                }
            }
        }

        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        mesh
    }
}

struct Player;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<Map>,
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

    let map_mesh = map.mesh();
    commands.spawn().insert_bundle(PbrBundle {
        mesh: meshes.add(map_mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.9, 0.1),
            ..Default::default()
        }),
        ..Default::default()
    });

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

fn exit(input: Res<Input<KeyCode>>, mut exit_event: EventWriter<bevy::app::AppExit>) {
    for key in input.get_just_pressed() {
        if *key == KeyCode::Escape {
            exit_event.send(bevy::app::AppExit);
        }
    }
}

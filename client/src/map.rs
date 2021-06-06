use bevy::prelude::*;
use heron::prelude::*;
use bevy::render::pipeline::{PipelineDescriptor, RenderPipeline};
use bevy::render::shader::ShaderStages;

#[derive(Clone, Copy, PartialEq, Eq)]
enum GroundType {
    Grass,
    Dirt,
    Water,
    Rock,
}

pub struct Map {
    dimensions: (u32, u32),
    ground: Vec<(GroundType, u32)>,
}

impl Map {
    fn build_map(dim: u32) -> Map {
        Map {
            dimensions: (dim, dim),
            ground: {
                let mut ground = Vec::with_capacity((dim * dim) as usize);
                let dim = dim as i32;
                for i in 0..dim {
                    for j in 0..dim {
                        let t = if (10 < i && i < 15 && (j - i) < 2) || (j % 10 < 2 && i % 7 < 3 && ((i + j) % 2 == 1)) {
                            GroundType::Dirt
                        } else if (i % 20 + ((j as f32).sin() as i32)) < 4 && (j % 40) <= i {
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
                            ((((j + i - 15) / 5) as f32).sin() * 4.0) as u32
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
    
    fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);
        let mut positions = Vec::with_capacity(self.ground.len());
        let mut normals = Vec::with_capacity(self.ground.len());
        let mut colors = Vec::with_capacity(self.ground.len());
        let mut indices = Vec::with_capacity(self.ground.len() * 3 * 2);
        let mut uv = Vec::with_capacity(self.ground.len());
        let (i, j) = self.dimensions;
        let half_i = i as f32 / 2.0;
        let half_j = j as f32 / 2.0;
        for x in 0..i {
            for y in 0..j {
                let (t, h) = self.ground[(x as usize) * (i as usize) + (y as usize)];
                positions.push([(x as f32) - half_i + 0.75, (h as f32) * 0.5, (y as f32) - half_j + 0.75]);

                colors.push(match t {
                    GroundType::Water => [0.0, 0.1, 0.8],
                    GroundType::Grass => [0.3, 0.6, 0.1],
                    GroundType::Rock => [0.5, 0.5, 0.6],
                    GroundType::Dirt => [0.5, 0.2, 0.2],
                });

                // TODO: this is probably very wrong
                let prev_h = if x > 0 && y < j - 1 {
                    let top_right_index = (x - 1) * i + y + 1;
                    self.ground[top_right_index as usize].1 as f32
                } else {
                    h as f32
                };
                let vec = Vec3::new(1.0, (h as f32) - prev_h, 1.0);
                let normal = vec.normalize().any_orthonormal_vector();
                normals.push([normal[0], normal[1], normal[2]]);

                uv.push([(x as f32) / (i as f32), (y as f32) / (j as f32)]);

                if x > 0 && y < (j - 1) {
                    let top_index = (x - 1) * i + y;
                    let right_index = x * i + y + 1;
                    let top_right_index = (x - 1) * i + y + 1;
                    let current_index = x * i + y;

                    indices.push(top_index);
                    indices.push(top_right_index);
                    indices.push(current_index);

                    indices.push(top_right_index);
                    indices.push(right_index);
                    indices.push(current_index);
                }
            }
        }

        mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        mesh
    }

    fn collision_shape(&self) -> Vec<Vec<f32>> {
        let (i, j) = self.dimensions;
        let mut res = Vec::with_capacity(i as usize);

        for x in 0..i {
            let mut line = Vec::with_capacity(j as usize);
            for y in 0..j {
                let (_, h) = self.ground[(x * i + y) as usize];
                line.push((h as f32) * 0.5);
            }
            res.push(line);
        }

        res
    }
}

pub struct ShaderHandles(Handle<Shader>, Handle<Shader>);

pub fn load_assets(mut commands: Commands, asset_serv: ResMut<AssetServer>) {
    asset_serv.watch_for_changes().unwrap();
    
    let vert_handle = asset_serv.load::<Shader, _>("shaders/map.vert");
    let frag_handle = asset_serv.load::<Shader, _>("shaders/map.frag");

    commands.insert_resource(ShaderHandles(vert_handle, frag_handle));
}

pub fn check_assets(
    mut state: ResMut<State<crate::State>>,
    asset_serv: ResMut<AssetServer>,
    shaders: ResMut<ShaderHandles>,
) {
    use bevy::asset::LoadState;
    let (vert_handle, frag_handle) = (shaders.0.clone(), shaders.1.clone());
    if dbg!(asset_serv.get_load_state(vert_handle.clone())) != LoadState::Loading && dbg!(asset_serv.get_load_state(frag_handle.clone())) != LoadState::Loading {
        state.set(crate::State::Main).unwrap();
    }
}

pub fn spawn_map(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut meshes: ResMut<Assets<Mesh>>,
    shaders: ResMut<ShaderHandles>,
) {
    let (vert_handle, frag_handle) = (shaders.0.clone(), shaders.1.clone());
    let pipeline = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: vert_handle.clone(),
        fragment: Some(frag_handle.clone()),
    }));

    let map = Map::build_map(50);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(map.mesh()),
        render_pipelines: RenderPipelines::from_pipelines(vec![
            RenderPipeline::new(pipeline),
        ]),
        ..Default::default()
    })
    .insert(CollisionShape::HeightField { heights: map.collision_shape(), size: Vec2::new(50.0, 50.0) })
    .insert(PhysicMaterial {
        restitution: 0.0,
        ..Default::default()
    })
    .insert(RigidBody::Static);
}

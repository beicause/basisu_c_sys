use bevy::{
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    math::Affine2,
    prelude::*,
    render::view::Hdr,
};
use bevy_basisu_loader::{BasisuLoaderPlugin, BasisuLoaderSettings};

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BasisuLoaderPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let skybox_handle = asset_server.load("gl_skybox_etc1s_cubemap_mips_12.basisu_ktx2");
    // camera
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::None,
        Msaa::Off,
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: skybox_handle.clone(),
            brightness: 1000.0,
            ..default()
        },
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("tough_uastc_ldr_4x4_mips_11.basisu_ktx2")),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(-1.0, 0.0, -2.5),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("desk_uastc_hdr_6x6_mips_10.basisu_ktx2")),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(1.0, 0.0, -5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.644 * 3.0, 0.874 * 3.0).mesh().build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            uv_transform: Affine2::from_scale(Vec2::new(2., 2.)),
            base_color_texture: Some(asset_server.load_with_settings(
                "desk_uastc_hdr_4x4_mips_10.basisu_ktx2",
                |s: &mut BasisuLoaderSettings| {
                    s.sampler =
                        bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                            address_mode_u: bevy::image::ImageAddressMode::Repeat,
                            address_mode_v: bevy::image::ImageAddressMode::Repeat,
                            ..Default::default()
                        })
                },
            )),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(3.0, 0.0, -3.0)
            * Transform::from_rotation(Quat::from_rotation_y(-45.0)),
    ));

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb_u8(210, 220, 240),
        brightness: 1.0,
        ..default()
    });
}

fn rotate_camera(
    mut query: Query<&mut Transform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let rotate = if keyboard_input.pressed(KeyCode::KeyQ) {
        0.05
    } else if keyboard_input.pressed(KeyCode::KeyE) {
        -0.05
    } else {
        0.0
    };
    for mut transform in &mut query {
        transform.rotate_y(rotate);
    }
}

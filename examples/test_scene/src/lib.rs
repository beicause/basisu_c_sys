use bevy::{
    asset::{AssetMetaCheck, HandleTemplate},
    camera::Hdr,
    core_pipeline::{Skybox, tonemapping::Tonemapping},
    ecs::template::OptionTemplate,
    feathers::{
        FeathersPlugins,
        controls::FeathersRadio,
        dark_theme::create_dark_theme,
        theme::{InheritableThemeTextColor, ThemeBackgroundColor, ThemedText, UiTheme},
        tokens,
    },
    log::LogPlugin,
    prelude::*,
    text::TextBounds,
    ui::{Checked, GlobalZIndex, widget::NodeImageMode},
    ui_widgets::{RadioGroup, ValueChange, radio_self_update},
};
use bevy_basisu_loader::{BasisuLoaderPlugin, BasisuLoaderSettings};

const IMAGE_PATH_ALPHA0: &str = "alpha0_etc1s_mips.basisu.ktx2";
const IMAGE_PATH_DESK1: &str = "desk_uastc_hdr_4x4.basisu.ktx2";
const IMAGE_PATH_DESK2: &str = "desk_uastc_hdr_6x6_mips.basisu.ktx2";
const IMAGE_PATH_KODIM20: &str = "kodim20_astc_ldr_8x8_mips.basisu.ktx2";
const IMAGE_PATH_SKYBOX: &str = "skybox_xuastc_ldr_8x8_cubemap_mips.basisu.ktx2";
const IMAGE_PATH_TOUGH: &str = "tough_uastc_ldr_4x4.basisu.ktx2";
const IMAGE_PATH_WIKIPEDIA: &str = "wikipedia_xuastc_ldr_8x8_mips.basisu.ktx2";
const SNAPSHOT_PATH_DESK: &str = "snapshots/encoder__desk_uastc_hdr_6x6_mips.snap.basisu.ktx2";
const SNAPSHOT_PATH_SKYBOX_ASTC: &str =
    "snapshots/encoder__skybox_astc_ldr_8x8_mips.snap.basisu.ktx2";
const SNAPSHOT_PATH_SKYBOX_XUASTC: &str =
    "snapshots/encoder__skybox_xuastc_ldr_4x4.snap.basisu.ktx2";
const ORIGINAL_PATH_ALPHA0: &str = "original_assets/alpha0.png";
const ORIGINAL_PATH_DESK: &str = "original_assets/Desk_fixed_6x6.exr";
const ORIGINAL_PATH_KODIM20: &str = "original_assets/kodim20.png";
const ORIGINAL_PATH_TOUGH: &str = "original_assets/tough_fixed.png";
const ORIGINAL_PATH_WIKIPEDIA: &str = "original_assets/wikipedia_fixed_6x6.png";
const ORIGINAL_PATH_SKYBOX_RIGHT: &str = "original_assets/skybox/right.jpg";
const ORIGINAL_PATH_SKYBOX_LEFT: &str = "original_assets/skybox/left.jpg";
const ORIGINAL_PATH_SKYBOX_TOP: &str = "original_assets/skybox/top.jpg";
const ORIGINAL_PATH_SKYBOX_BOTTOM: &str = "original_assets/skybox/bottom.jpg";
const ORIGINAL_PATH_SKYBOX_FRONT: &str = "original_assets/skybox/front.jpg";
const ORIGINAL_PATH_SKYBOX_BACK: &str = "original_assets/skybox/back.jpg";

/// Marks the root entity of the currently active content scene (grid or skybox).
/// Scene switching despawns every entity with this component, then spawns the new
/// scene with it. Declared inside each switchable scene's `bsn!` so every root of a
/// multi-root scene is marked (bsn requires `Clone` on component entries).
#[derive(Component, Default, Clone)]
struct SceneRoot;

/// Handles for assets that are loaded once and reused, so scene switching never
/// re-issues loads or re-transcodes textures:
/// - `alpha0`/`desk2` need special `BasisuLoaderSettings` (`load_builder`),
/// - the three cube maps are swapped at runtime by the skybox scene's radio group.
/// Plain-path assets (grid cells, originals) resolve through bevy's per-path handle
/// cache at spawn time, so they need no explicit caching.
#[derive(Resource)]
struct LoadedAssets {
    alpha0: Handle<Image>,
    desk2: Handle<Image>,
    skybox: Handle<Image>,
    skybox_astc: Handle<Image>,
    skybox_xuastc: Handle<Image>,
}

/// Marks a grid cell whose fixed slot size and image fit are computed at runtime so the
/// whole 8-column grid fits on one screen regardless of window size.
#[derive(Component, Default, Clone)]
struct GridCell;

/// Grid layout: 8 columns across, cells sized as if 4 rows fill the window height.
const GRID_COLS: f32 = 8.0;
const GRID_ROWS: f32 = 4.0;
const GRID_GAP: f32 = 8.0;
const GRID_MARGIN: f32 = 12.0;
/// Bottom padding that clears the persistent button bar (absolute, bottom-left).
const GRID_BOTTOM: f32 = 64.0;
/// Image inset inside its cell.
const CELL_PADDING: f32 = 6.0;
/// Vertical space reserved for the (possibly two-line) file-name label.
const LABEL_SPACE: f32 = 40.0;

/// One-shot sizing pass for `GridCell` cells: gives every cell its 1/8 × 1/4 window slot
/// and scales each image down (never up) to fit the slot while preserving aspect ratio.
/// Runs every frame until the image assets are loaded, then removes `GridCell`.
fn fit_cell_images(
    mut commands: Commands,
    mut cells: Query<(Entity, &mut Node, &Children), With<GridCell>>,
    mut image_nodes: Query<(&mut Node, &ImageNode), Without<GridCell>>,
    mut text_bounds: Query<&mut TextBounds, Without<GridCell>>,
    images: Res<Assets<Image>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let cell_w = (window.width() - 2.0 * GRID_MARGIN - (GRID_COLS - 1.0) * GRID_GAP) / GRID_COLS;
    let cell_h =
        (window.height() - GRID_BOTTOM - GRID_MARGIN - (GRID_ROWS - 1.0) * GRID_GAP) / GRID_ROWS;
    let mut fitted = Vec::new();
    for (entity, mut node, children) in &mut cells {
        node.width = Val::Px(cell_w);
        node.height = Val::Px(cell_h);
        let mut ready = true;
        for child in children.iter() {
            if let Ok((mut child_node, image_node)) = image_nodes.get_mut(child) {
                let Some(image) = images.get(&image_node.image) else {
                    // Image asset not loaded yet; retry next frame.
                    ready = false;
                    continue;
                };
                let size = image.size();
                let scale = ((cell_w - 2.0 * CELL_PADDING) / size.x as f32)
                    .min((cell_h - LABEL_SPACE) / size.y as f32)
                    .min(1.0);
                child_node.width = Val::Px(size.x as f32 * scale);
                child_node.height = Val::Px(size.y as f32 * scale);
                info!(
                    "fit: cell {:.0}x{:.0}, image {:.0}x{:.0} -> {:.0}x{:.0} (scale {:.3})",
                    cell_w,
                    cell_h,
                    size.x as f32,
                    size.y as f32,
                    size.x as f32 * scale,
                    size.y as f32 * scale,
                    scale,
                );
            } else if let Ok(mut bounds) = text_bounds.get_mut(child) {
                bounds.width = Some(cell_w - 2.0 * CELL_PADDING);
            }
        }
        if ready {
            fitted.push(entity);
        }
    }
    for entity in fitted {
        commands.entity(entity).remove::<GridCell>();
    }
}

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..Default::default()
                })
                .set(LogPlugin {
                    filter: "bevy_basisu_loader=debug,wgpu=warn".to_string(),
                    ..Default::default()
                }),
        )
        .add_plugins((BasisuLoaderPlugin, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate_camera, fit_cell_images))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load special-handling assets once and cache their handles; scene switches reuse them.
    let alpha0 = load_alpha0(&asset_server);
    let desk2 = load_desk2(&asset_server);
    let skybox = asset_server.load(IMAGE_PATH_SKYBOX);
    let skybox_astc = asset_server.load(SNAPSHOT_PATH_SKYBOX_ASTC);
    let skybox_xuastc = asset_server.load(SNAPSHOT_PATH_SKYBOX_XUASTC);
    commands.insert_resource(LoadedAssets {
        alpha0: alpha0.clone(),
        desk2: desk2.clone(),
        skybox,
        skybox_astc,
        skybox_xuastc,
    });
    // Persistent UI layer (scene-switch radio bar), not part of either switchable scene.
    commands.spawn_scene(ui_scene());
    // Scene 1 (default): image grid. Scene 2 (skybox) is reached via the Skybox radio.
    commands.spawn_scene(images_scene(alpha0, desk2));
}

fn load_alpha0(asset_server: &AssetServer) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|s: &mut BasisuLoaderSettings| {
            s.channel_type_hint = bevy_basisu_loader::sys::extra::ChannelType::Rg;
        })
        .load(IMAGE_PATH_ALPHA0)
}

fn load_desk2(asset_server: &AssetServer) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|s: &mut BasisuLoaderSettings| {
            s.force_transcode_target = Some(bevy_basisu_loader::sys::TranscodeTargetFormat::Rgb9e5);
        })
        .load(IMAGE_PATH_DESK2)
}

fn ui_scene() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16.0),
            left: px(16.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(16.0),
        }
        GlobalZIndex(1)
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        Children [
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(16.0),
                }
                RadioGroup
                on(radio_self_update)
                Children [
                    (
                        @FeathersRadio {
                            @caption: bsn! {
                                Node {
                                    padding: px(8.0),
                                    align_items: AlignItems::Center,
                                }
                                Children [ (Text("Images") TextFont { font_size: px(22.0) } ThemedText) ]
                            },
                        }
                        Checked
                        on(|_change: On<ValueChange<bool>>, mut commands: Commands, loaded: Res<LoadedAssets>, roots: Query<Entity, With<SceneRoot>>| {
                            for root in &roots {
                                commands.entity(root).despawn();
                            }
                            commands.spawn_scene(images_scene(loaded.alpha0.clone(), loaded.desk2.clone()));
                        })
                    ),
                    (
                        @FeathersRadio {
                            @caption: bsn! {
                                Node {
                                    padding: px(8.0),
                                    align_items: AlignItems::Center,
                                }
                                Children [ (Text("Skybox") TextFont { font_size: px(22.0) } ThemedText) ]
                            },
                        }
                        on(|_change: On<ValueChange<bool>>, mut commands: Commands, roots: Query<Entity, With<SceneRoot>>| {
                            for root in &roots {
                                commands.entity(root).despawn();
                            }
                            commands.spawn_scene_list(skybox_scene());
                        })
                    ),
                ]
            ),
        ]
    }
}

/// Scene 1: image grid. Every `.basisu.ktx2` asset (including the 3 encoder snapshots)
/// plus every original asset, each original adjacent to its compressed variants, laid out
/// in a single grid of 8 columns (up to 4 rows) that fits on one screen; images are
/// downscaled (never upscaled) to fit their cell with aspect ratio preserved, and every
/// cell is labeled with its file name. Cell sizes are set at runtime by `fit_cell_images`.
fn images_scene(alpha0: Handle<Image>, desk2: Handle<Image>) -> impl Scene {
    bsn! {
        Camera2d
        // The grid shows HDR assets (desk .exr, HDR ktx2): an HDR target plus no
        // tonemapping keeps the float values intact instead of clamping them to LDR.
        Hdr
        template_value(Tonemapping::None)
        SceneRoot
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Center,
            column_gap: px(GRID_GAP),
            row_gap: px(GRID_GAP),
            padding: UiRect::new(
                px(GRID_MARGIN),
                px(GRID_MARGIN),
                px(GRID_MARGIN),
                px(GRID_BOTTOM),
            ),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        InheritableThemeTextColor(tokens::TEXT_MAIN)
        Children [
            (image_cell(ORIGINAL_PATH_ALPHA0, "alpha0.png")),
            (image_cell_handle(alpha0, IMAGE_PATH_ALPHA0)),
            (image_cell(ORIGINAL_PATH_DESK, "Desk_fixed_6x6.exr")),
            (image_cell(IMAGE_PATH_DESK1, IMAGE_PATH_DESK1)),
            (image_cell_handle(desk2, IMAGE_PATH_DESK2)),
            (image_cell(SNAPSHOT_PATH_DESK, "encoder__desk_uastc_hdr_6x6_mips.snap.basisu.ktx2")),
            (image_cell(ORIGINAL_PATH_KODIM20, "kodim20.png")),
            (image_cell(IMAGE_PATH_KODIM20, IMAGE_PATH_KODIM20)),
            (image_cell(ORIGINAL_PATH_TOUGH, "tough_fixed.png")),
            (image_cell(IMAGE_PATH_TOUGH, IMAGE_PATH_TOUGH)),
            (image_cell(ORIGINAL_PATH_WIKIPEDIA, "wikipedia_fixed_6x6.png")),
            (image_cell(IMAGE_PATH_WIKIPEDIA, IMAGE_PATH_WIKIPEDIA)),
            (image_cell(ORIGINAL_PATH_SKYBOX_RIGHT, "skybox/right.jpg")),
            (image_cell(ORIGINAL_PATH_SKYBOX_LEFT, "skybox/left.jpg")),
            (image_cell(ORIGINAL_PATH_SKYBOX_TOP, "skybox/top.jpg")),
            (image_cell(ORIGINAL_PATH_SKYBOX_BOTTOM, "skybox/bottom.jpg")),
            (image_cell(ORIGINAL_PATH_SKYBOX_FRONT, "skybox/front.jpg")),
            (image_cell(ORIGINAL_PATH_SKYBOX_BACK, "skybox/back.jpg")),
            // Cube map ktx2 files cannot be displayed in an `ImageNode` (the UI material
            // requires a D2 texture view; cube views fail validation and crash the app),
            // so they get a label-only cell.
            (label_cell(IMAGE_PATH_SKYBOX)),
            (label_cell(SNAPSHOT_PATH_SKYBOX_ASTC)),
            (label_cell(SNAPSHOT_PATH_SKYBOX_XUASTC)),
        ]
    }
}

/// Scene 2: skybox with a camera-control hint (top-left) and a radio group (top-right)
/// that switches between the three cube-map ktx2 assets. Three scene roots, each marked
/// `SceneRoot` so the whole scene despawns when switching back to the grid:
/// 1. the camera (Camera3d + Skybox), 2. the hint UI root, 3. the cubemap picker UI root.
/// UI roots must have no parent (no `ChildOf`), which is why the overlays are roots here
/// instead of children of the camera.
fn skybox_scene() -> impl SceneList {
    let skybox_image = OptionTemplate::from(HandleTemplate::from(IMAGE_PATH_SKYBOX));
    let backdrop = Color::srgba(0.0, 0.0, 0.0, 0.45);
    bsn_list! {
        (
            Camera3d::default()
            template_value(Tonemapping::None)
            Transform::from_xyz(0.0, 0.0, 3.0)
            Skybox {
                image: skybox_image,
                brightness: 1000.0,
            }
            SceneRoot
        ),
        (
            // Camera-control hint (only shown in this scene).
            Node {
                position_type: PositionType::Absolute,
                top: px(12.0),
                left: px(12.0),
                padding: UiRect::axes(px(10.0), px(6.0)),
                border_radius: px(6.0),
            }
            GlobalZIndex(1)
            BackgroundColor(backdrop)
            InheritableThemeTextColor(tokens::TEXT_MAIN)
            SceneRoot
            Children [
                (
                    Text("Q/E: rotate camera")
                    TextFont { font_size: px(16.0) }
                    ThemedText
                )
            ]
        ),
        (
            // Cube-map picker: the three cube-map ktx2 assets share the same skybox
            // shader, only the texture handle changes.
            Node {
                position_type: PositionType::Absolute,
                top: px(12.0),
                right: px(12.0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6.0),
                padding: UiRect::all(px(8.0)),
                border_radius: px(6.0),
            }
            GlobalZIndex(1)
            BackgroundColor(backdrop)
            RadioGroup
            on(radio_self_update)
            SceneRoot
            Children [
                (
                    @FeathersRadio {
                        @caption: bsn! {
                            Node {
                                padding: px(4.0),
                                align_items: AlignItems::Center,
                            }
                            Children [ (Text("xuastc 8x8") TextFont { font_size: px(16.0) } ThemedText) ]
                        },
                    }
                    Checked
                    on(|_change: On<ValueChange<bool>>, mut skybox: Single<&mut Skybox>, loaded: Res<LoadedAssets>| {
                        skybox.image = Some(loaded.skybox.clone());
                    })
                ),
                (
                    @FeathersRadio {
                        @caption: bsn! {
                            Node {
                                padding: px(4.0),
                                align_items: AlignItems::Center,
                            }
                            Children [ (Text("astc 8x8 snap") TextFont { font_size: px(16.0) } ThemedText) ]
                        },
                    }
                    on(|_change: On<ValueChange<bool>>, mut skybox: Single<&mut Skybox>, loaded: Res<LoadedAssets>| {
                        skybox.image = Some(loaded.skybox_astc.clone());
                    })
                ),
                (
                    @FeathersRadio {
                        @caption: bsn! {
                            Node {
                                padding: px(4.0),
                                align_items: AlignItems::Center,
                            }
                            Children [ (Text("xuastc 4x4 snap") TextFont { font_size: px(16.0) } ThemedText) ]
                        },
                    }
                    on(|_change: On<ValueChange<bool>>, mut skybox: Single<&mut Skybox>, loaded: Res<LoadedAssets>| {
                        skybox.image = Some(loaded.skybox_xuastc.clone());
                    })
                ),
            ]
        )
    }
}

/// A grid cell for assets that cannot be rendered in an `ImageNode` (cube map ktx2):
/// just the file name label, centered in the runtime-sized cell.
fn label_cell(label: &'static str) -> impl Scene {
    bsn! {
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        GridCell
        Children [
            (Text(label) TextFont { font_size: px(12.0) } TextBounds { width: None, height: None } ThemedText),
        ]
    }
}

/// A grid cell loading an image by asset path string (bsn resolves it to a Handle at
/// spawn time via HandleTemplate) with a file-name label underneath.
fn image_cell(path: &'static str, label: &'static str) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(4.0),
        }
        GridCell
        Children [
            (ImageNode {
                image: path,
                image_mode: NodeImageMode::Stretch,
            }),
            (Text(label) TextFont { font_size: px(12.0) } TextBounds { width: None, height: None } ThemedText),
        ]
    }
}

/// A grid cell taking a preloaded handle (needed for the special BasisuLoaderSettings
/// loads: alpha0 Rg channel hint, desk2 Rgb9e5 transcode).
fn image_cell_handle(handle: Handle<Image>, label: &'static str) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(4.0),
        }
        GridCell
        Children [
            (ImageNode {
                image: handle,
                image_mode: NodeImageMode::Stretch,
            }),
            (Text(label) TextFont { font_size: px(12.0) } TextBounds { width: None, height: None } ThemedText),
        ]
    }
}

fn rotate_camera(
    mut query: Query<&mut Transform, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let rotate = if keyboard_input.pressed(KeyCode::KeyQ)
        || keyboard_input.pressed(KeyCode::ArrowLeft)
    {
        0.05
    } else if keyboard_input.pressed(KeyCode::KeyE) || keyboard_input.pressed(KeyCode::ArrowRight) {
        -0.05
    } else {
        0.0
    };
    for mut transform in &mut query {
        transform.rotate_y(rotate);
    }
}

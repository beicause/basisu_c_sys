use std::collections::HashMap;

use bevy::{
    asset::{AssetMetaCheck, HandleTemplate, LoadState, RenderAssetUsages},
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
    image::ImageSampler,
    input::mouse::{MouseScrollUnit, MouseWheel},
    log::LogPlugin,
    picking::hover::HoverMap,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDataOrder, TextureDescriptor, TextureDimension, TextureUsages,
        TextureViewDescriptor, TextureViewDimension,
    },
    scene::EntityScene,
    text::LineBreak,
    ui::{
        Checked, GlobalZIndex,
        widget::{ImageNodeSize, NodeImageMode},
    },
    ui_widgets::{RadioGroup, ValueChange, radio_self_update},
    window::WindowResolution,
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
/// - `grid` caches every plain-path grid asset (`GridEntry::Path`, loaded once in
///   `setup`), so re-spawning the grid scene reuses the same handles,
/// - the three cube-map ktx2 assets are swapped at runtime by the skybox scene's
///   radio group,
/// - `face_cubemap` is the uncompressed cubemap stitched from the six skybox face
///   jpgs by `build_face_cubemap` (None until that system finishes).
#[derive(Resource)]
struct LoadedAssets {
    alpha0: Handle<Image>,
    desk2: Handle<Image>,
    skybox: Handle<Image>,
    skybox_astc: Handle<Image>,
    skybox_xuastc: Handle<Image>,
    face_cubemap: Option<Handle<Image>>,
    /// Plain-path grid assets, keyed by their asset path.
    grid: HashMap<&'static str, Handle<Image>>,
}

/// The six skybox face images, in cubemap layer order (+X, -X, +Y, -Y, +Z, -Z) — the
/// same order `examples/test_processor/assets/skybox.pack.ron` feeds to the encoder.
const SKYBOX_FACES: [&str; 6] = [
    ORIGINAL_PATH_SKYBOX_RIGHT,
    ORIGINAL_PATH_SKYBOX_LEFT,
    ORIGINAL_PATH_SKYBOX_TOP,
    ORIGINAL_PATH_SKYBOX_BOTTOM,
    ORIGINAL_PATH_SKYBOX_FRONT,
    ORIGINAL_PATH_SKYBOX_BACK,
];

/// Grid layout: `GRID_COLS` equal columns; every image row is exactly `100 / GRID_ROWS`
/// percent of the grid's height (labels sit in their own auto-sized rows below each
/// image), so `GRID_ROWS` image rows are visible on screen at once — fractions allowed,
/// e.g. 3.5 means three and a half rows — and the remaining cells are reached by
/// scrolling (the clipped viewport + translated `GridContent`, driven by
/// [`scroll_grid`]).
const GRID_COLS: f32 = 5.0;
const GRID_ROWS: f32 = 3.5;
const GRID_GAP: f32 = 8.0;
const GRID_MARGIN: f32 = 12.0;
/// Bottom padding that clears the persistent button bar (absolute, bottom-left).
const GRID_BOTTOM: f32 = 64.0;
/// Dark translucent backdrop behind the floating UI panels, so light text and radio
/// outlines stay legible over both the dark grid and the bright skybox.
const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.65);

/// Per-app state for [`build_face_cubemap`]: the face handles we started loading
/// (so the loads are issued exactly once) and a sticky failure flag.
#[derive(Default)]
struct BuildCubemapState {
    handles: Vec<Handle<Image>>,
    failed: bool,
}

/// Loads the six skybox face jpgs and stitches them into a single uncompressed
/// cubemap `Image`: 6 array layers in (+X, -X, +Y, -Y, +Z, -Z) order, viewed as
/// [`TextureViewDimension::Cube`] — the same layering `skybox.pack.ron` feeds to the
/// basisu encoder, so this cubemap matches the ktx2 ones visually. The result is
/// added to `Assets<Image>` and cached in `LoadedAssets.face_cubemap` for the skybox
/// scene's "uncompressed" radio. Runs once per app start; no-op afterwards.
fn build_face_cubemap(
    mut state: Local<BuildCubemapState>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut loaded: ResMut<LoadedAssets>,
) {
    if loaded.face_cubemap.is_some() || state.failed {
        return;
    }
    // Issue the loads once; `asset_server.load` dedupes against the grid's copies.
    if state.handles.is_empty() {
        state.handles = SKYBOX_FACES.map(|path| asset_server.load(path)).to_vec();
        return;
    }
    // Wait until every face is loaded (or stop forever on a hard failure).
    for handle in &state.handles {
        match asset_server.load_state(handle) {
            LoadState::Loaded => {}
            LoadState::Failed(err) => {
                error!("failed to load skybox face for cubemap: {err}");
                state.failed = true;
                return;
            }
            LoadState::NotLoaded | LoadState::Loading => return,
        }
    }
    let mut faces = Vec::with_capacity(6);
    for handle in &state.handles {
        faces.push(images.get(handle).unwrap());
    }
    // All faces must share one 2D size and format to be stitched into layers.
    let size = faces[0].texture_descriptor.size;
    let format = faces[0].texture_descriptor.format;
    let layer_bytes = size.width as usize * size.height as usize * 4;
    let mut data = Vec::with_capacity(6 * layer_bytes);
    for image in &faces {
        let descriptor = &image.texture_descriptor;
        if descriptor.size != size
            || descriptor.dimension != TextureDimension::D2
            || descriptor.format != format
            || image.data.as_deref().is_none_or(|d| d.len() != layer_bytes)
        {
            error!(
                "skybox faces are not uniform ({}x{} {:?} vs {}x{} {:?}); \
                 skipping uncompressed cubemap",
                descriptor.size.width,
                descriptor.size.height,
                descriptor.format,
                size.width,
                size.height,
                format,
            );
            state.failed = true;
            return;
        }
        data.extend_from_slice(image.data.as_deref().unwrap());
    }
    let cubemap = Image {
        data: Some(data),
        data_order: TextureDataOrder::default(),
        texture_descriptor: TextureDescriptor {
            label: Some("skybox faces cubemap"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::Default,
        texture_view_descriptor: Some(TextureViewDescriptor {
            label: Some("skybox faces cubemap view"),
            dimension: Some(TextureViewDimension::Cube),
            array_layer_count: Some(6),
            mip_level_count: Some(1),
            ..default()
        }),
        asset_usage: RenderAssetUsages::RENDER_WORLD,
        copy_on_resize: false,
    };
    let handle = images.add(cubemap);
    loaded.face_cubemap = Some(handle);
    info!("built uncompressed skybox cubemap ({size:?}, {} layers)", 6);
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
                        // Workaround for https://github.com/bevyengine/bevy/issues/25306:
                        // on touch screens /
                        // device emulation the backend scale factor is != 1.0, and
                        // bevy 0.19 UI renders 2x and picking coordinates misalign.
                        // Forcing 1.0 lays the UI correctly,
                        // though touch input is still incorrect.
                        resolution: WindowResolution::default().with_scale_factor_override(1.0),
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
        .add_systems(
            Update,
            (
                rotate_camera,
                build_face_cubemap,
                fit_grid_images,
                size_grid_rows,
                scroll_grid,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load special-handling assets once and cache their handles; scene switches reuse them.
    let alpha0 = load_alpha0(asset_server.as_ref());
    let desk2 = load_desk2(asset_server.as_ref());
    let skybox = asset_server.load(IMAGE_PATH_SKYBOX);
    let skybox_astc = asset_server.load(SNAPSHOT_PATH_SKYBOX_ASTC);
    let skybox_xuastc = asset_server.load(SNAPSHOT_PATH_SKYBOX_XUASTC);
    let mut loaded = LoadedAssets {
        alpha0: alpha0.clone(),
        desk2: desk2.clone(),
        skybox,
        skybox_astc,
        skybox_xuastc,
        face_cubemap: None,
        grid: HashMap::new(),
    };
    // Cache every plain-path grid asset once, so switching back to the grid scene
    // reuses the same handles instead of re-resolving the paths at spawn time.
    for entry in GRID_ENTRIES {
        if let GridEntry::Path(path) = *entry {
            loaded.grid.insert(path, asset_server.load(path));
        }
    }
    // Scene 1 (default): image grid. Scene 2 (skybox) is reached via the Skybox radio.
    let grid_scene = images_scene(&loaded);
    commands.insert_resource(loaded);
    // Persistent UI layer (scene-switch radio bar), not part of either switchable scene.
    commands.spawn_scene(ui_scene());
    commands.spawn_scene(grid_scene);
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
            bottom: px(12.0),
            left: px(12.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(16.0),
            padding: UiRect::all(px(8.0)),
            border_radius: px(8.0),
        }
        GlobalZIndex(1)
        BackgroundColor(OVERLAY_BG)
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
                                Children [ (Text("Images") TextFont { font_size: px(20.0) } ThemedText) ]
                            },
                        }
                        Checked
                        on(|_change: On<ValueChange<bool>>, mut commands: Commands, loaded: Res<LoadedAssets>, roots: Query<Entity, With<SceneRoot>>| {
                            for root in &roots {
                                commands.entity(root).despawn();
                            }
                            commands.spawn_scene(images_scene(&loaded));
                        })
                    ),
                    (
                        @FeathersRadio {
                            @caption: bsn! {
                                Node {
                                    padding: px(8.0),
                                    align_items: AlignItems::Center,
                                }
                                Children [ (Text("Skybox") TextFont { font_size: px(20.0) } ThemedText) ]
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

/// One cell of the Scene 1 grid, in display order. The grid is fully described by the
/// [`GRID_ENTRIES`] table below — add, remove, or reorder entries to reconfigure it
/// (combined with `GRID_COLS` / `GRID_ROWS` for the layout).
#[derive(Clone, Copy)]
enum GridEntry {
    /// Load this asset path with default loader settings.
    Path(&'static str),
    /// Use a handle cached in [`LoadedAssets`] (needs special `BasisuLoaderSettings`).
    Handle(HandleKind),
}

/// Which cached handle a [`GridEntry::Handle`] cell uses.
#[derive(Clone, Copy)]
enum HandleKind {
    Alpha0,
    Desk2,
}

/// Marks a Scene-1 grid image entity; [`fit_grid_images`] sizes it to fit its cell
/// while keeping the source aspect ratio.
#[derive(Component, Default, Clone)]
struct GridImage;

/// Scene 1's grid cells, left-to-right / top-to-bottom: every compressed `.basisu.ktx2`
/// with its original next to it (alpha0, desk, kodim20, tough, wikipedia, skybox faces).
/// The cube-map ktx2 assets are not in the grid — they are switchable in the skybox scene.
const GRID_ENTRIES: &[GridEntry] = &[
    GridEntry::Path(ORIGINAL_PATH_ALPHA0),
    GridEntry::Handle(HandleKind::Alpha0),
    GridEntry::Path(ORIGINAL_PATH_DESK),
    GridEntry::Path(IMAGE_PATH_DESK1),
    GridEntry::Handle(HandleKind::Desk2),
    GridEntry::Path(SNAPSHOT_PATH_DESK),
    GridEntry::Path(ORIGINAL_PATH_KODIM20),
    GridEntry::Path(IMAGE_PATH_KODIM20),
    GridEntry::Path(ORIGINAL_PATH_TOUGH),
    GridEntry::Path(IMAGE_PATH_TOUGH),
    GridEntry::Path(ORIGINAL_PATH_WIKIPEDIA),
    GridEntry::Path(IMAGE_PATH_WIKIPEDIA),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_RIGHT),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_LEFT),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_TOP),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_BOTTOM),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_FRONT),
    GridEntry::Path(ORIGINAL_PATH_SKYBOX_BACK),
];

/// File name portion of an asset path; used as the cell label.
fn file_name(path: &'static str) -> &'static str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Builds one cell per [`GRID_ENTRIES`] entry, in order. Returns a `SceneList`
/// so it can be spliced into the grid's `Children` with `{ … }`. Each entry is two
/// placed grid items: the image on row `2k+1`, its label on the auto-sized row `2k+2`.
/// Every cell uses a cached handle from [`LoadedAssets`]: plain paths from the `grid`
/// cache (filled in `setup`), the two special-settings loads from `alpha0`/`desk2`.
fn grid_cells(loaded: &LoadedAssets) -> impl SceneList {
    let mut cells: Vec<Box<dyn SceneList>> = Vec::new();
    let cols = GRID_COLS as i16;
    for (i, entry) in GRID_ENTRIES.iter().enumerate() {
        let i = i as i16;
        let col = i % cols + 1;
        let image_row = (i / cols) * 2 + 1;
        let label_row = image_row + 1;
        let cell: Box<dyn SceneList> = match entry {
            GridEntry::Path(path) => Box::new(image_cell(
                loaded
                    .grid
                    .get(path)
                    .expect("grid asset cached in setup")
                    .clone(),
                file_name(path),
                col,
                image_row,
                label_row,
            )),
            GridEntry::Handle(HandleKind::Alpha0) => Box::new(image_cell(
                loaded.alpha0.clone(),
                file_name(IMAGE_PATH_ALPHA0),
                col,
                image_row,
                label_row,
            )),
            GridEntry::Handle(HandleKind::Desk2) => Box::new(image_cell(
                loaded.desk2.clone(),
                file_name(IMAGE_PATH_DESK2),
                col,
                image_row,
                label_row,
            )),
        };
        cells.push(cell);
    }
    cells
}

/// Scene 1: image grid. Every entry of [`GRID_ENTRIES`] gets one cell in a CSS grid:
/// `GRID_COLS` equal `fr` columns; image rows are sized by [`size_grid_rows`] so that
/// `GRID_ROWS` image rows fit the visible height (labels live in auto-sized rows under
/// each image). Rows beyond the viewport are reached by scrolling: the grid lives
/// inside a clipped viewport and [`scroll_grid`] translates the `GridContent` wrapper
/// (bevy 0.19's `ScrollPosition` mechanism cannot scroll grid containers — taffy
/// reports a grid's content size as its largest item, so the scroll range is always
/// empty). Every image is contain-fitted into its cell by [`fit_grid_images`], so it
/// never exceeds the cell and keeps its original aspect ratio.
fn images_scene(loaded: &LoadedAssets) -> impl Scene {
    // Grid tracks: GRID_COLS equal columns; one image row per visible row (px, sized
    // by `size_grid_rows` from the viewport height), each followed by an auto-sized
    // label row (explicit tracks, so labels never change the image row height).
    let columns = vec![RepeatedGridTrack::flex(GRID_COLS as u16, 1.0)];
    let image_row_count = (GRID_ENTRIES.len() as f32 / GRID_COLS).ceil() as usize;
    let mut rows: Vec<RepeatedGridTrack> = Vec::with_capacity(image_row_count * 2);
    for _ in 0..image_row_count {
        rows.push(GridTrack::px::<RepeatedGridTrack>(0.0)); // placeholder; sized below
        rows.push(GridTrack::auto::<RepeatedGridTrack>());
    }
    let auto_rows = vec![GridTrack::auto()];
    bsn! {
        Camera2d
        // The grid shows HDR assets (desk .exr, HDR ktx2): an HDR target plus no
        // tonemapping keeps the float values intact instead of clamping them to LDR.
        Hdr
        template_value(Tonemapping::None)
        SceneRoot
        // Clipped viewport: rows below the visible area are hidden and revealed by
        // translating the GridContent wrapper below (`scroll_grid`).
        GridViewport
        Node {
            width: percent(100),
            height: percent(100),
            // Flex-start so the GridContent wrapper below sizes to its content
            // (the grid's full height) instead of stretching to the viewport —
            // the wrapper's height IS the scrollable extent.
            align_items: AlignItems::FlexStart,
            overflow: Overflow::clip(),
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
            (
                // Scrollable content: translated by -scroll in `scroll_grid`.
                GridContent
                ScrollStart(0.0)
                Node {
                    width: percent(100),
                    position_type: PositionType::Relative,
                    top: px(0.0),
                }
                Children [
                    (
                        // The actual CSS grid; its height is its content (auto), so
                        // the wrapper's ComputedNode gives the scrollable extent.
                        GridRoot
                        Node {
                            width: percent(100),
                            display: Display::Grid,
                            grid_template_columns: columns,
                            grid_template_rows: rows,
                            grid_auto_rows: auto_rows,
                            // Top-aligned so overflowing rows scroll into view from
                            // the bottom instead of being clipped symmetrically.
                            align_content: AlignContent::FlexStart,
                            column_gap: px(GRID_GAP),
                            row_gap: px(GRID_GAP),
                        }
                        Children [
                            { grid_cells(loaded) }
                        ]
                    )
                ]
            )
        ]
    }
}

/// Scene 2: skybox with a camera-control hint (top-left) and a radio group (top-right)
/// that switches between the three cube-map ktx2 assets and the runtime-stitched
/// uncompressed cubemap. Three scene roots, each marked
/// `SceneRoot` so the whole scene despawns when switching back to the grid:
/// 1. the camera (Camera3d + Skybox),
/// 2. the hint UI root,
/// 3. the cubemap picker UI root.
///
/// UI roots must have no parent (no `ChildOf`), which is why the overlays are roots here
/// instead of children of the camera.
fn skybox_scene() -> impl SceneList {
    let skybox_image = OptionTemplate::from(HandleTemplate::from(IMAGE_PATH_SKYBOX));
    let backdrop = OVERLAY_BG;
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
            SceneRoot
            Children [
                (
                    Text("Q/E (Left/Right): rotate camera")
                    TextFont { font_size: px(16.0) }
                    // Match the radio captions' effective rendering: the caption
                    // wrapper `Node` has no `ThemedText`, so the theme's
                    // `With<ThemedText>` propagation filter stops before the caption
                    // text and it never receives the RADIO_TEXT color — it renders
                    // bevy's default `TextColor::WHITE`. An explicit white here
                    // reproduces that exactly; without it the hint root's TEXT_MAIN
                    // token would propagate LIGHT_GRAY_1 and the hint would look
                    // grayer than the captions.
                    TextColor(Color::WHITE)
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
                (
                    // Cubemap stitched at runtime from the six skybox face jpgs by
                    // `build_face_cubemap` (uncompressed). No-op until it is ready.
                    @FeathersRadio {
                        @caption: bsn! {
                            Node {
                                padding: px(4.0),
                                align_items: AlignItems::Center,
                            }
                            Children [ (Text("uncompressed") TextFont { font_size: px(16.0) } ThemedText) ]
                        },
                    }
                    on(|_change: On<ValueChange<bool>>, mut skybox: Single<&mut Skybox>, loaded: Res<LoadedAssets>| {
                        if let Some(handle) = loaded.face_cubemap.clone() {
                            skybox.image = Some(handle);
                        }
                    })
                ),
            ]
        )
    }
}

/// A grid cell: the image on its `100/GRID_ROWS`-percent row, the file-name label in
/// the auto-sized row beneath it. `col`/`image_row`/`label_row` are 1-based grid
/// lines. The image keeps its own aspect ratio and is contain-fitted into the cell
/// by [`fit_grid_images`] (largest aspect-preserving box, centered); the
/// overflow-hidden wrapper only guards sub-pixel rounding. The label uses a
/// viewport-relative (`Vw`) font and wraps at any character (`LineBreak::AnyCharacter`),
/// so it never exceeds the column width. The handle comes from [`LoadedAssets`] —
/// every grid asset is cached in `setup`, including the special
/// `BasisuLoaderSettings` loads (alpha0 Rg channel hint, desk2 Rgb9e5 transcode).
fn image_cell(
    handle: Handle<Image>,
    label: &'static str,
    col: i16,
    image_row: i16,
    label_row: i16,
) -> impl SceneList {
    let col = GridPlacement::start(col);
    let image_row = GridPlacement::start(image_row);
    let label_row = GridPlacement::start(label_row);
    (
        // Image row item: `fit_grid_images` sets the image's explicit px size to the
        // largest aspect-preserving box in the cell (`width: percent(100)` is only
        // the pre-fit fallback while the texture is still loading).
        EntityScene(bsn! {
            Node {
                overflow: Overflow::hidden(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                grid_row: image_row,
                grid_column: col,
            }
            Children [
                (
                    ImageNode {
                        image: handle,
                        image_mode: NodeImageMode::Auto,
                    }
                    Node {
                        width: percent(100),
                    }
                    GridImage
                )
            ]
        }),
        // Label row item: centers the file name under the image; the item stretches
        // to the grid column, so the text wraps within the column (file names have no
        // spaces, so `AnyCharacter` breaks at any char). Font size is viewport-
        // relative (`Vw`), so it scales with the window; wrapping keeps the label
        // inside the cell at any size.
        EntityScene(bsn! {
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                grid_row: label_row,
                grid_column: col,
            }
            Children [
                (
                    Text(label)
                    TextFont { font_size: FontSize::Vw(1.0) }
                    TextLayout { linebreak: LineBreak::AnyCharacter }
                    ThemedText
                )
            ]
        }),
    )
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

/// Marks the clipped viewport root of the grid scene; used to measure the visible
/// area (see [`scroll_grid`]).
#[derive(Component, Default, Clone)]
struct GridViewport;

/// Marks the scrollable content wrapper (translated by [`scroll_grid`]).
#[derive(Component, Default, Clone)]
struct GridContent;

/// Marks the CSS grid node inside [`GridContent`]; [`size_grid_rows`] sizes its
/// image rows from the viewport height.
#[derive(Component, Default, Clone)]
struct GridRoot;

/// Drag-scroll state on the [`GridContent`] wrapper: the scroll offset captured at
/// `Pointer<DragStart>`, so the drag delta maps 1:1 to the scroll offset (same
/// pattern as bevy 0.19's `ui/scroll_and_overflow/drag_to_scroll` example). Works
/// for touch and mouse drag alike (picking emits `Pointer` events for both).
#[derive(Component, Default, Clone)]
struct ScrollStart(f32);

/// Mouse-wheel distance per `MouseScrollUnit::Line`, in logical pixels.
const SCROLL_LINE_HEIGHT: f32 = 40.0;

/// Sizes the grid's image rows so `GRID_ROWS` of them fit the viewport's visible
/// height (the CSS-grid equivalent of the old `percent(100 / GRID_ROWS)` tracks,
/// but in px so the grid's auto height equals its content extent — required for
/// [`scroll_grid`] to measure the scroll range). Writes only on change.
/// All math is in logical px (`ComputedNode` is physical; `GridTrack::px` rescales
/// by the scale factor at layout time, so converting here avoids double-scaling).
fn size_grid_rows(
    mut q_grid: Query<&mut Node, With<GridRoot>>,
    viewport: Single<&ComputedNode, With<GridViewport>>,
) {
    let Ok(mut grid) = q_grid.single_mut() else {
        return;
    };
    let viewport_logical = viewport.size() * viewport.inverse_scale_factor();
    let visible_h = (viewport_logical.y - GRID_MARGIN - GRID_BOTTOM).max(1.0);
    let image_h = visible_h / GRID_ROWS;
    let image_row_count = (GRID_ENTRIES.len() as f32 / GRID_COLS).ceil() as usize;
    let mut rows: Vec<RepeatedGridTrack> = Vec::with_capacity(image_row_count * 2);
    for _ in 0..image_row_count {
        rows.push(GridTrack::px::<RepeatedGridTrack>(image_h));
        rows.push(GridTrack::auto::<RepeatedGridTrack>());
    }
    if grid.grid_template_rows != rows {
        grid.grid_template_rows = rows;
    }
}

/// Scrolls the grid by translating the `GridContent` wrapper, driven by two inputs:
/// - wheel: `MouseWheel` deltas, same sign and `Line` scaling as bevy 0.19's
///   `ui/scroll_and_overflow/scroll` example; the pointer must hover the grid
///   subtree (walked up via `ChildOf`);
/// - drag (touch or mouse): `Pointer<DragStart>` / `Pointer<Drag>` messages
///   (`Pointer` is both a `Message` and an `EntityEvent`), same pattern as bevy
///   0.19's `ui/scroll_and_overflow/drag_to_scroll` example — the offset at drag
///   start is captured in [`ScrollStart`], then the drag delta maps 1:1.
///
/// The scroll range is `content height − visible height`, clamped; no-op when the
/// content fits or the pointer/drag target is outside the grid (e.g. over the
/// persistent button bar).
#[expect(clippy::too_many_arguments, reason = "It is a complex function")]
fn scroll_grid(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    mut drag_start_reader: MessageReader<Pointer<DragStart>>,
    mut drag_reader: MessageReader<Pointer<Drag>>,
    hover_map: Res<HoverMap>,
    ui_scale: Res<UiScale>,
    mut q_content: Query<(&mut Node, &ComputedNode, Entity, &mut ScrollStart), With<GridContent>>,
    viewport: Single<&ComputedNode, With<GridViewport>>,
    q_parent: Query<&ChildOf>,
) {
    let Ok((mut content_node, content, content_entity, mut scroll_start)) = q_content.single_mut()
    else {
        return;
    };
    // `ComputedNode` sizes are physical; convert to logical px so the `Val::Px`
    // translation (`top`) is not double-scaled by the layout scale factor.
    let viewport_logical = viewport.size() * viewport.inverse_scale_factor();
    let visible_h = viewport_logical.y - GRID_MARGIN - GRID_BOTTOM;
    let content_logical = content.size() * content.inverse_scale_factor();
    let max_scroll = (content_logical.y - visible_h).max(0.0);
    if max_scroll <= 0.0 {
        return; // content fits; nothing to scroll
    }
    let current = match content_node.top {
        Val::Px(v) => -v,
        _ => 0.0,
    };
    // Is `entity` inside the grid subtree (the wrapper or any descendant)?
    let in_grid = |mut entity: Entity| -> bool {
        loop {
            if entity == content_entity {
                return true;
            }
            match q_parent.get(entity) {
                Ok(parent) => entity = parent.0,
                Err(_) => return false,
            }
        }
    };
    // DragStart: capture the offset so the drag delta maps 1:1 (touch or mouse).
    for drag_start in drag_start_reader.read() {
        if in_grid(drag_start.entity) {
            scroll_start.0 = current;
        }
    }
    // Drag: follow the pointer.
    for drag in drag_reader.read() {
        if in_grid(drag.entity) {
            let next = (scroll_start.0 - drag.distance.y / ui_scale.0).clamp(0.0, max_scroll);
            if (next - current).abs() > 0.01 {
                content_node.top = Val::Px(-next);
            }
        }
    }
    // Wheel: only when the pointer hovers the grid subtree.
    let mut delta_y = 0.0;
    for mouse_wheel in mouse_wheel_reader.read() {
        let delta = -mouse_wheel.y;
        delta_y += if mouse_wheel.unit == MouseScrollUnit::Line {
            delta * SCROLL_LINE_HEIGHT
        } else {
            delta
        };
    }
    if delta_y != 0.0
        && hover_map
            .values()
            .flat_map(|map| map.keys().copied())
            .any(in_grid)
    {
        let next = (current + delta_y).clamp(0.0, max_scroll);
        if (next - current).abs() > 0.01 {
            content_node.top = Val::Px(-next);
        }
    }
}

/// Contain-fits each grid image inside its cell: reads the cell size from the
/// wrapper's `ComputedNode` and the source texture size from `ImageNodeSize`
/// (auto-maintained by bevy's `update_image_content_size_system`), then sets the
/// image node's `width`/`height` in logical px so the image fills the largest
/// aspect-preserving box inside the cell. No-op while the texture is unloaded or
/// the wrapper is not laid out; converges one frame after either becomes ready;
/// re-fits automatically on window resize. The skybox scene has no `GridImage`
/// entities, so it is a no-op there.
fn fit_grid_images(
    mut q_image: Query<(&ChildOf, &mut Node, &ImageNodeSize), With<GridImage>>,
    q_wrapper: Query<&ComputedNode>,
) {
    for (parent, mut node, image_size) in &mut q_image {
        let Ok(wrapper) = q_wrapper.get(parent.0) else {
            continue;
        };
        let src = image_size.size().as_vec2();
        // `ComputedNode` is physical; convert to logical px so `Val::Px` (rescaled
        // by the layout's scale factor) lands on the intended size at any DPR.
        let cell = wrapper.size() * wrapper.inverse_scale_factor();
        if src.x <= 0.0 || src.y <= 0.0 || cell.x <= 0.0 || cell.y <= 0.0 {
            continue; // texture not loaded yet, or cell not laid out yet
        }
        let scale = (cell / src).min_element();
        let fit = src * scale;
        if node.width != Val::Px(fit.x) || node.height != Val::Px(fit.y) {
            node.width = Val::Px(fit.x);
            node.height = Val::Px(fit.y);
        }
    }
}

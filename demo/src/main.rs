use self::{setup::*, visual_effects::*, yarn_slinger_integration::*};
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy::window::PresentMode;
use bevy_editor_pls::EditorPlugin;
use bevy_sprite3d::Sprite3dPlugin;
use bevy_yarn_slinger::prelude::*;
use bevy_yarn_slinger_example_ui::prelude::*;

mod easing;
mod setup;
mod visual_effects;
mod yarn_slinger_integration;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                watch_for_changes: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Yarn Slinger Story Demo".into(),
                    resolution: (800., 600.).into(),
                    present_mode: PresentMode::AutoVsync,
                    prevent_default_event_handling: false,
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugin(EditorPlugin::new())
    .add_plugin(
        YarnSlingerPlugin::with_yarn_files(vec!["story.yarn"]).with_localizations(Localizations {
            base_localization: "en-US".into(),
            translations: vec!["de-CH".into()],
            file_generation_mode: FileGenerationMode::DEVELOPMENT_ON_SUPPORTED_PLATFORMS,
        }),
    )
    .add_plugin(ExampleYarnSlingerUiPlugin::new())
    .add_plugin(Sprite3dPlugin)
    .add_systems((
        setup.on_startup(),
        spawn_dialogue_runner.run_if(resource_added::<YarnProject>()),
        adapt_materials.run_if(any_with_component::<SceneInstance>()),
        spawn_sprites.run_if(sprites_have_loaded),
        rotate_sprite,
        handle_fade.run_if(resource_exists::<FadeCurtainAlpha>()),
        move_camera.run_if(resource_exists::<CameraMovement>()),
    ))
    .add_systems(
        (change_speaker, bob_speaker)
            .chain()
            .after(ExampleYarnSlingerUiSystemSet),
    )
    .run();
}

#[derive(Resource)]
struct Sprites {
    ferris_neutral: Handle<Image>,
    ferris_happy: Handle<Image>,
    clippy: Handle<Image>,
}

fn sprites_have_loaded(sprites: Res<Sprites>, asset_server: Res<AssetServer>) -> bool {
    asset_server.get_load_state(&sprites.ferris_neutral) == LoadState::Loaded
        && asset_server.get_load_state(&sprites.ferris_happy) == LoadState::Loaded
        && asset_server.get_load_state(&sprites.clippy) == LoadState::Loaded
}

const FERRIS_TRANSLATION: Vec3 = Vec3::new(-1.3, 0.9, 0.35);
const CLIPPY_TRANSLATION: Vec3 = Vec3::new(1.5, 0.94, -0.5);
const CAMERA_TRANSLATION: Vec3 = Vec3::new(-1.7, 1.4, 1.8);
const SECOND_ACT_CAMERA_TRANSLATION: Vec3 = Vec3::new(-2.0, 1.4, 1.8);

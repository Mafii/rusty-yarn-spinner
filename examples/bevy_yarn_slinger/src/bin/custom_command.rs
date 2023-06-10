use bevy::prelude::*;
use bevy_yarn_slinger::prelude::*;
use bevy_yarn_slinger_example_ui::prelude::*;

// For comments about the setup, see hello_world.rs
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(YarnSlingerPlugin::with_yarn_files(vec![
            "custom_command.yarn",
        ]))
        .add_plugin(ExampleYarnSlingerUiPlugin::new())
        .add_systems((
            setup_camera.on_startup(),
            spawn_dialogue_runner.run_if(resource_added::<YarnProject>()),
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    let mut dialogue_runner = project.default_dialogue_runner().unwrap();
    // Add our custom commands to the dialogue runner
    dialogue_runner
        .command_registrations_mut()
        .register_command("insert_resource", insert_resource)
        .register_command("read_resource", read_resource);
    dialogue_runner.start();
    commands.spawn(dialogue_runner);
}

#[derive(Resource)]
struct SomethingAddedByYarnSlinger {
    name: String,
    age: f32,
}

// Commands are valid Bevy systems with inputs (and optional outputs).
// The `In` param will determine the yarn signature. This function can thus be called like
// `<<insert_resource "Bob" 42>>` in Yarn.
fn insert_resource(In((name, age)): In<(&str, f32)>, mut commands: Commands) {
    commands.insert_resource(SomethingAddedByYarnSlinger {
        name: name.to_string(),
        age,
    });
}

// Commands with no inputs have the unit type (`()`) as their input.
// This function can thus be called like `<<read_resource>>` in Yarn.
fn read_resource(_: In<()>, previously_added_resource: Res<SomethingAddedByYarnSlinger>) {
    println!(
        "{} is {} years old",
        previously_added_resource.name, previously_added_resource.age
    );
}

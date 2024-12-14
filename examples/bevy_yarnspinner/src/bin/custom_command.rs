use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner_example_dialogue_view::prelude::*;

// For comments about the setup, see hello_world.rs
fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        YarnSpinnerPlugin::new(),
        ExampleYarnSpinnerDialogueViewPlugin::new(),
    ))
    .add_systems(Startup, setup_camera)
    .add_systems(
        Update,
        spawn_dialogue_runner.run_if(resource_added::<YarnProject>),
    )
    .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    let mut dialogue_runner = project.create_dialogue_runner();
    // Add our custom commands to the dialogue runner
    dialogue_runner
        .commands_mut()
        .add_command("insert_resource", insert_resource)
        .add_command("update_resource", update_resource)
        .add_command("read_resource", read_resource);
    dialogue_runner.start_node("CustomCommand");
    commands.spawn(dialogue_runner);
}

#[derive(Resource)]
struct SomethingAddedByYarnSpinner {
    name: String,
    age: f32,
}

// Commands are valid Bevy systems with inputs (and optional outputs).
// The `In` param will determine the Yarn signature. This function can thus be called like
// `<<insert_resource "Bob" 42>>` in Yarn.
fn insert_resource(In((name, age)): In<(String, f32)>, mut commands: Commands) {
    commands.insert_resource(SomethingAddedByYarnSpinner { name, age });
}

// Commands with no inputs have the unit type (`()`) as their input.
// This function can thus be called like `<<read_resource>>` in Yarn.
fn read_resource(_: In<()>, previously_added_resource: Res<SomethingAddedByYarnSpinner>) {
    println!(
        "{} is {} years old",
        previously_added_resource.name, previously_added_resource.age
    );
}

// Commands may also take arguments as `Option<_>` which allows commands with optional arguments.
// This function can be called either like:
//     `<<update_resource "Bob">>`
// or
//     `<<update_resource "Bob" 42>>`
fn update_resource(
    In((name, age)): In<(String, Option<f32>)>,
    mut resource: ResMut<SomethingAddedByYarnSpinner>,
) {
    resource.name = name;
    if let Some(age) = age {
        resource.age = age;
    }
}

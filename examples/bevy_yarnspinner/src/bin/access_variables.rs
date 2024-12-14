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
        (
            spawn_dialogue_runner.run_if(resource_added::<YarnProject>),
            print_yarn_variable.run_if(any_with_component::<DialogueRunner>),
        ),
    )
    .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    let mut dialogue_runner = project.create_dialogue_runner();
    dialogue_runner.start_node("AccessVariables");
    commands.spawn(dialogue_runner);
}

fn print_yarn_variable(dialogue_runner: Query<&DialogueRunner>) {
    let dialogue_runner = dialogue_runner.single();
    let value = dialogue_runner.variable_storage().get("$foo");
    if let Ok(value) = value {
        println!("foo: {value}");
    }
}

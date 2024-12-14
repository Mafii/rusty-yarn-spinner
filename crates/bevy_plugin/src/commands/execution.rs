use crate::commands::UntypedYarnCommand;
use crate::dialogue_runner::DialogueExecutionSystemSet;
use crate::events::ExecuteCommandEvent;
use crate::prelude::*;
use bevy::ecs::event::EventCursor;
use bevy::prelude::*;

pub(crate) fn command_execution_plugin(app: &mut App) {
    app.add_systems(
        Update,
        execute_commands
            .after(DialogueExecutionSystemSet)
            .in_set(YarnSpinnerSystemSet),
    );
}

fn execute_commands(world: &mut World, mut cursor: Local<EventCursor<ExecuteCommandEvent>>) {
    let events = clone_events(world, &mut cursor);
    for event in events {
        let Some(mut command) = clone_command(world, &event) else {
            continue;
        };
        let params = event.command.parameters;
        let task_finished_indicator = command.call(params, world);
        if !task_finished_indicator.is_finished() {
            get_dialogue_runner_mut(world, event.source).add_command_task(task_finished_indicator);
        }
    }
}

fn clone_events(
    world: &World,
    cursor: &mut EventCursor<ExecuteCommandEvent>,
) -> Vec<ExecuteCommandEvent> {
    let events = world.resource::<Events<ExecuteCommandEvent>>();
    cursor.read(events).cloned().collect()
}

fn clone_command(
    world: &mut World,
    event: &ExecuteCommandEvent,
) -> Option<Box<dyn UntypedYarnCommand>> {
    let dialogue_runner = get_dialogue_runner(world, event.source);
    let command_name = event.command.name.as_str();
    dialogue_runner
        .commands
        .get(command_name)
        .map(|command| command.clone_box())
}

fn get_dialogue_runner(world: &mut World, entity: Entity) -> &DialogueRunner {
    let mut dialogue_runners = world.query::<&DialogueRunner>();
    let dialogue_runner = dialogue_runners.get(world, entity).unwrap();
    dialogue_runner
}

fn get_dialogue_runner_mut(world: &mut World, entity: Entity) -> Mut<DialogueRunner> {
    let mut dialogue_runners = world.query::<&mut DialogueRunner>();
    let dialogue_runner = dialogue_runners.get_mut(world, entity).unwrap();
    dialogue_runner
}

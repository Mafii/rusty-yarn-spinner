use crate::setup::{spawn_options, DialogueNode, OptionButton, OptionsNode, UiRootNode};
use crate::typewriter::{self, Typewriter, TypewriterFinishedEvent};
use crate::ExampleYarnSpinnerDialogueViewSystemSet;
use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::{PrimaryWindow, SystemCursorIcon};
use bevy::winit::cursor::CursorIcon;
use bevy_yarnspinner::{events::*, prelude::*};

pub(crate) fn option_selection_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            create_options.run_if(resource_added::<OptionSelection>),
            show_options,
            select_option
                .run_if(resource_exists::<OptionSelection>.and(any_with_component::<PrimaryWindow>))
                .before(typewriter::despawn),
            despawn_options,
        )
            .chain()
            .after(YarnSpinnerSystemSet)
            .in_set(ExampleYarnSpinnerDialogueViewSystemSet),
    )
    .add_event::<HasSelectedOptionEvent>();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Event)]
struct HasSelectedOptionEvent;

#[derive(Debug, Clone, PartialEq, Default, Resource)]
pub(crate) struct OptionSelection {
    options: Vec<DialogueOption>,
}

impl OptionSelection {
    pub fn from_option_set<'a>(options: impl IntoIterator<Item = &'a DialogueOption>) -> Self {
        let options = options
            .into_iter()
            .filter(|o| o.is_available)
            .cloned()
            .collect();
        Self { options }
    }
}

fn create_options(
    option_selection: Res<OptionSelection>,
    mut commands: Commands,
    children: Query<&Children>,
    mut options_node: Query<(Entity, &mut Node, &mut Visibility), With<OptionsNode>>,
    mut root_visibility: Query<&mut Visibility, (With<UiRootNode>, Without<OptionsNode>)>,
) {
    let (entity, mut node, mut visibility) = options_node.single_mut();
    node.display = Display::Flex;
    *visibility = Visibility::Hidden;
    if children.iter_descendants(entity).next().is_none() {
        *root_visibility.single_mut() = Visibility::Inherited;
        let mut entity_commands = commands.entity(entity);
        spawn_options(&mut entity_commands, &option_selection.options);
    }
}

fn show_options(
    mut typewriter_finished_event: EventReader<TypewriterFinishedEvent>,
    mut options_node: Query<&mut Visibility, With<OptionsNode>>,
) {
    for _event in typewriter_finished_event.read() {
        let mut visibility = options_node.single_mut();
        *visibility = Visibility::Inherited;
    }
}

fn select_option(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    typewriter: Res<Typewriter>,
    mut buttons: Query<
        (&Interaction, &OptionButton, &Children),
        (With<Button>, Changed<Interaction>),
    >,
    mut dialogue_runners: Query<&mut DialogueRunner>,
    text_entities: Query<Entity, (With<Text>, Without<DialogueNode>)>,
    mut text_writer: TextUiWriter,
    option_selection: Res<OptionSelection>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut selected_option_event: EventWriter<HasSelectedOptionEvent>,
) {
    if !typewriter.is_finished() {
        return;
    }

    let mut selection = None;
    let key_to_option: HashMap<_, _> = NUMBER_KEYS
        .into_iter()
        .zip(NUMPAD_KEYS)
        .zip(option_selection.options.iter().map(|option| option.id))
        .collect();
    for ((num_key, numpad_key), option) in key_to_option {
        if keys.just_pressed(num_key) || keys.just_pressed(numpad_key) {
            selection = Some(option);
            break;
        }
    }

    for (interaction, button, children) in buttons.iter_mut() {
        let (color, icon) = match *interaction {
            Interaction::Pressed if selection.is_none() => {
                selection = Some(button.0);
                (css::TOMATO.into(), SystemCursorIcon::Default)
            }
            Interaction::Hovered => (Color::WHITE, SystemCursorIcon::Pointer),
            _ => (css::TOMATO.into(), SystemCursorIcon::Default),
        };
        commands
            .entity(windows.single())
            .insert(CursorIcon::System(icon));
        let text_entity = children
            .iter()
            .find(|&e| text_entities.contains(*e))
            .unwrap();
        *text_writer.color(*text_entity, 2) = TextColor(color);
    }
    let has_selected_id = selection.is_some();
    if let Some(id) = selection {
        for mut dialogue_runner in dialogue_runners.iter_mut() {
            dialogue_runner.select_option(id).unwrap();
        }
    }
    if has_selected_id {
        selected_option_event.send(HasSelectedOptionEvent);
    }
}

fn despawn_options(
    mut has_selected_option_event: EventReader<HasSelectedOptionEvent>,
    mut dialogue_complete_event: EventReader<DialogueCompleteEvent>,
    mut commands: Commands,
    mut options_node: Query<(Entity, &mut Node, &mut Visibility), With<OptionsNode>>,
    mut dialogue_node_text: Query<&mut Text, With<DialogueNode>>,
    mut root_visibility: Query<&mut Visibility, (With<UiRootNode>, Without<OptionsNode>)>,
) {
    let should_despawn =
        !has_selected_option_event.is_empty() || !dialogue_complete_event.is_empty();
    if !should_despawn {
        return;
    }
    has_selected_option_event.clear();
    dialogue_complete_event.clear();
    commands.remove_resource::<OptionSelection>();
    let (entity, mut node, mut visibility) = options_node.single_mut();
    commands.entity(entity).despawn_descendants();
    node.display = Display::None;
    *visibility = Visibility::Hidden;
    *dialogue_node_text.single_mut() = Text::default();
    *root_visibility.single_mut() = Visibility::Hidden;
}

const NUMBER_KEYS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

const NUMPAD_KEYS: [KeyCode; 9] = [
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
];

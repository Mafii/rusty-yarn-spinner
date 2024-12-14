use crate::assets::{font_handle, image_handle};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;

pub(crate) fn ui_setup_plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

/// Marker for the [`Node`] that is the root of the UI
#[derive(Debug, Default, Component)]
pub struct UiRootNode;

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueNode;

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueNameNode;

#[derive(Debug, Default, Component)]
pub(crate) struct DialogueContinueNode;

#[derive(Debug, Default, Component)]
pub(crate) struct OptionsNode;

#[derive(Debug, Component)]
pub(crate) struct OptionButton(pub OptionId);

fn setup(mut commands: Commands) {
    // root node
    commands
        .spawn((
            fmt_name("root"),
            Node {
                display: Display::Grid,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_content: AlignContent::End,
                justify_content: JustifyContent::SpaceAround,
                grid_auto_flow: GridAutoFlow::Row,
                grid_template_columns: vec![RepeatedGridTrack::minmax(
                    1,
                    MinTrackSizingFunction::Auto,
                    MaxTrackSizingFunction::Px(DIALOG_WIDTH),
                )],
                // In the web, `GridTrack::auto()` should already work like this,
                // but it seems like Bevy disagrees?
                // If you remove this line in the future and everything looks the same, go ahead!
                grid_auto_rows: vec![GridTrack::min_content()],
                ..default()
            },
            Visibility::Hidden,
            UiRootNode,
        ))
        .with_children(|parent| {
            parent.spawn((
                fmt_name("name"),
                Text::default(),
                text_style::name(),
                Node {
                    margin: UiRect {
                        left: Val::Px(TEXT_BORDER_HORIZONTAL / 2.0),
                        bottom: Val::Px(-8.0),
                        ..default()
                    },
                    ..default()
                },
                ZIndex(1),
                DialogueNameNode,
                Label,
            ));

            parent
                .spawn((
                    fmt_name("dialogue"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect {
                            top: Val::Px(TEXT_BORDER_TOP),
                            bottom: Val::Px(TEXT_BORDER_BOTTOM),
                            left: Val::Px(TEXT_BORDER_HORIZONTAL),
                            right: Val::Px(TEXT_BORDER_HORIZONTAL),
                        },
                        ..default()
                    },
                    BackgroundColor(Color::BLACK.with_alpha(0.8)),
                    BorderRadius::all(Val::Px(20.0)),
                ))
                .with_children(|parent| {
                    // Dialog itself
                    parent.spawn((
                        fmt_name("text"),
                        Text::default(),
                        text_style::standard(),
                        style::standard(),
                        DialogueNode,
                        Label,
                    ));
                })
                .with_children(|parent| {
                    // Options
                    parent.spawn((
                        fmt_name("options"),
                        Node {
                            display: Display::None,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexStart,
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },
                        Visibility::Hidden,
                        OptionsNode,
                    ));
                });

            parent.spawn((
                fmt_name("continue indicator"),
                ImageNode {
                    // 27 x 27 pixels
                    image: image_handle::CONTINUE_INDICATOR,
                    ..default()
                },
                Node {
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    margin: UiRect {
                        top: Val::Px(-18.),
                        bottom: Val::Px(25.),
                        ..default()
                    },
                    ..default()
                },
                ZIndex(1),
                Visibility::Hidden,
                DialogueContinueNode,
            ));
        });
}

fn fmt_name(name: &str) -> Name {
    Name::new(format!("Yarn Spinner example dialogue view node: {name}"))
}

pub(crate) fn create_dialog_text(
    text: impl Into<String>,
    invisible: impl Into<String>,
) -> [(TextSpan, TextFont, TextColor); 2] {
    [
        (
            TextSpan(text.into()),
            text_style::standard().0,
            text_style::standard().1,
        ),
        (
            TextSpan(invisible.into()),
            text_style::standard().0,
            TextColor(Color::NONE),
        ),
    ]
}

pub(crate) fn spawn_options<'a, T>(entity_commands: &mut EntityCommands, options: T)
where
    T: IntoIterator<Item = &'a DialogueOption>,
    <T as IntoIterator>::IntoIter: 'a,
{
    entity_commands.with_children(|parent| {
        for (i, option) in options.into_iter().enumerate() {
            parent
                .spawn((
                    fmt_name("option button"),
                    Node {
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                    ImageNode::default().with_color(Color::NONE),
                    Button,
                    OptionButton(option.id),
                ))
                .with_children(|parent| {
                    let spans = [
                        (TextSpan(format!("{}: ", i + 1)), text_style::option_id()),
                        (
                            TextSpan(option.line.text.clone()),
                            text_style::option_text(),
                        ),
                    ];

                    parent
                        .spawn((
                            fmt_name("option text"),
                            Text::default(),
                            style::options(),
                            Label,
                        ))
                        .with_children(|parent| {
                            parent.spawn(spans[0].clone());
                            parent.spawn(spans[1].clone());
                        });
                });
        }
    });
}

const DIALOG_WIDTH: f32 = 800.0 * 0.8;
const TEXT_BORDER_HORIZONTAL: f32 = 120.0;
const TEXT_BORDER_TOP: f32 = 30.0;
const TEXT_BORDER_BOTTOM: f32 = TEXT_BORDER_TOP + 10.0;

mod style {
    use super::*;
    pub(crate) fn standard() -> Node {
        Node {
            max_width: Val::Px(DIALOG_WIDTH - 2.0 * TEXT_BORDER_HORIZONTAL),
            ..default()
        }
    }
    pub(crate) fn options() -> Node {
        const INDENT_MODIFIER: f32 = 1.0;
        Node {
            margin: UiRect::horizontal(Val::Px((INDENT_MODIFIER - 1.0) * TEXT_BORDER_HORIZONTAL)),
            max_width: Val::Px(DIALOG_WIDTH - 2.0 * INDENT_MODIFIER * TEXT_BORDER_HORIZONTAL),
            ..default()
        }
    }
}

mod text_style {
    use super::*;
    use bevy::color::palettes::css;
    pub(crate) fn standard() -> (TextFont, TextColor) {
        (
            TextFont {
                font: font_handle::MEDIUM,
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
        )
    }
    pub(crate) fn name() -> (TextFont, TextColor) {
        (
            TextFont {
                font: font_handle::MEDIUM,
                font_size: 18.0,
                ..standard().0
            },
            standard().1,
        )
    }

    pub(crate) fn option_id() -> (TextFont, TextColor) {
        (
            TextFont {
                font: font_handle::MEDIUM,
                ..option_text().0
            },
            TextColor(css::ALICE_BLUE.into()),
        )
    }

    pub(crate) fn option_text() -> (TextFont, TextColor) {
        (
            TextFont {
                font_size: 18.0,
                ..standard().0
            },
            TextColor(css::TOMATO.into()),
        )
    }
}

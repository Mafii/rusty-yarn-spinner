use bevy::ecs::event::EventCursor;
use bevy::prelude::*;
use bevy_yarnspinner::events::*;

#[derive(Debug, Default)]
pub struct EventAsserter {
    pub present_line_cursor: EventCursor<PresentLineEvent>,
    pub present_options_cursor: EventCursor<PresentOptionsEvent>,
    pub dialogue_start_cursor: EventCursor<DialogueStartEvent>,
    pub dialogue_complete_cursor: EventCursor<DialogueCompleteEvent>,
    pub node_start_cursor: EventCursor<NodeStartEvent>,
    pub node_complete_cursor: EventCursor<NodeCompleteEvent>,
    pub line_hints_cursor: EventCursor<LineHintsEvent>,
    pub execute_command_cursor: EventCursor<ExecuteCommandEvent>,
}

impl EventAsserter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_events(&mut self, app: &mut App) {
        self.present_line_cursor
            .clear(app.world().resource::<Events<PresentLineEvent>>());
        self.present_options_cursor
            .clear(app.world().resource::<Events<PresentOptionsEvent>>());
        self.dialogue_start_cursor
            .clear(app.world().resource::<Events<DialogueStartEvent>>());
        self.dialogue_complete_cursor
            .clear(app.world().resource::<Events<DialogueCompleteEvent>>());
        self.node_start_cursor
            .clear(app.world().resource::<Events<NodeStartEvent>>());
        self.node_complete_cursor
            .clear(app.world().resource::<Events<NodeCompleteEvent>>());
        self.line_hints_cursor
            .clear(app.world().resource::<Events<LineHintsEvent>>());
        self.execute_command_cursor
            .clear(app.world().resource::<Events<ExecuteCommandEvent>>());
    }
}

#[macro_export]
macro_rules! get_cursor {
    ($asserter:ident, PresentLineEvent) => {
        &mut $asserter.present_line_cursor
    };
    ($asserter:ident, PresentOptionsEvent) => {
        &mut $asserter.present_options_cursor
    };
    ($asserter:ident, DialogueStartEvent) => {
        &mut $asserter.dialogue_start_cursor
    };
    ($asserter:ident, DialogueCompleteEvent) => {
        &mut $asserter.dialogue_complete_cursor
    };
    ($asserter:ident, NodeStartEvent) => {
        &mut $asserter.node_start_cursor
    };
    ($asserter:ident, NodeCompleteEvent) => {
        &mut $asserter.node_complete_cursor
    };
    ($asserter:ident, LineHintsEvent) => {
        &mut $asserter.line_hints_cursor
    };
    ($asserter:ident, ExecuteCommandEvent) => {
        &mut $asserter.execute_command_cursor
    };
}

#[macro_export]
macro_rules! assert_events {
    ($asserter:ident, $app:ident contains [$($event:ident $((n = $num:expr))? $(with $pred:expr)?) ,* $(,)?]) => {
        $(
            { assert_events!($asserter, $app contains $event $((n = $num))? $(with $pred)?); }
        )*
    };
    ($asserter:ident, $app:ident contains $event:ident $(with $pred:expr)?) => {
        assert_events!($asserter, $app contains $event (n = 1) $(with $pred)?);
    };
    ($asserter:ident, $app:ident contains $event:ident (n = $num:expr) $(with $pred:expr)?) => {
        let events = $app.world().resource::<bevy::prelude::Events<$event>>();
        let cursor = $crate::get_cursor!($asserter, $event);
        let events: Vec<&$event> = cursor.read(&events).collect();
        assert_eq!($num, events.len(), "Expected {} events of type {}, but found {}: {events:#?}", stringify!($num), stringify!($event), events.len());
        $(
            {
                fn get_pred() -> impl Fn(&$event) -> bool {
                    $pred
                }
                let pred = get_pred();
                let actual = events.into_iter().next().unwrap();
                assert!(pred(actual), "Expected event of type {} to fulfill predicate {}, but found {:#?}", stringify!($event), stringify!($pred), actual);
            }
        )?
    };
}

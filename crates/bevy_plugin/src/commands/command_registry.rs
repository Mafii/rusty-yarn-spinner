use crate::commands::command_wrapping::YarnCommandWrapper;
use crate::commands::UntypedYarnCommand;
use crate::prelude::*;
use bevy::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

pub(crate) mod wait;

pub(crate) fn command_registry_plugin(app: &mut App) {
    app.add_plugins(wait::wait_command_plugin);
}

#[derive(Debug, PartialEq, Eq, Default)]
/// A registry of commands that can be called from Yarn after they have been added via [`YarnCommands::add_command`].
/// You can get access to an instance of this struct with [`DialogueRunner::commands`] and [`DialogueRunner::commands_mut`].
///
/// If a command "add_player" with the parameters "name" and "age" has been registered, it can be called from Yarn like this:
/// ```text
/// <<add_player "John" 42>>
/// ```
pub struct YarnCommands(pub(crate) InnerRegistry);

type InnerRegistry = HashMap<Cow<'static, str>, Box<dyn UntypedYarnCommand>>;

impl Extend<<InnerRegistry as IntoIterator>::Item> for YarnCommands {
    fn extend<T: IntoIterator<Item = <InnerRegistry as IntoIterator>::Item>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl IntoIterator for YarnCommands {
    type Item = <InnerRegistry as IntoIterator>::Item;
    type IntoIter = <InnerRegistry as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl YarnCommands {
    /// Instantiates a new, empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new method to the registry. Commands are valid Bevy systems with input and output.
    ///
    /// See the documentation of [`YarnCommand`] for more information about which methods are allowed.
    pub fn add_command<Marker, F>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        command: F,
    ) -> &mut Self
    where
        Marker: 'static,
        F: YarnCommand<Marker> + 'static + Clone,
    {
        let name = name.into();
        let wrapped = YarnCommandWrapper::from(command);
        self.0.insert(name, Box::new(wrapped));
        self
    }

    /// Iterates over all registered commands.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &(dyn UntypedYarnCommand))> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_ref(), value.as_ref()))
    }

    /// Returns `true` if the registry contains a command with the given name.
    pub fn contains_key(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Returns a reference to the command with the given name, if it exists.
    pub fn get(&self, name: &str) -> Option<&(dyn UntypedYarnCommand)> {
        self.0.get(name).map(|f| f.as_ref())
    }

    /// Returns a mutable reference to the command with the given name, if it exists.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn UntypedYarnCommand)> {
        self.0.get_mut(name).map(|f| f.as_mut())
    }

    /// Iterates over all registered command names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|key| key.as_ref())
    }

    /// Iterates over all registered commands.
    pub fn commands(&self) -> impl Iterator<Item = &(dyn UntypedYarnCommand)> {
        self.0.values().map(|value| value.as_ref())
    }

    /// Returns the number of registered commands.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the registry contains no commands.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Constructs an instance of [`YarnCommands`] with the builtin commands `wait` and `stop`.
    /// - `stop`: Stops the execution of the dialogue.
    /// - `wait`: Waits for the given amount of seconds before continuing the dialogue. Note that this does not block and that Bevy will continue updating as normal in the meantime.
    pub fn builtin_commands() -> Self {
        let mut commands = Self::default();

        commands.add_command("wait", |In(duration): In<f32>, mut wait: ResMut<Wait>| {
            wait.add(Duration::from_secs_f32(duration))
        });

        #[allow(clippy::unused_unit)] // Needed for 2024 edition
        commands.add_command("stop", |_: In<()>| -> () {
            unreachable!("The stop command is a compiler builtin and is thus not callable");
        });

        commands
    }
}

/// Convenience macro for creating a [`YarnCommands`] instance with the given commands.
/// ## Example
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_yarnspinner::prelude::*;
/// # use bevy_yarnspinner::yarn_commands;
///
/// let commands = yarn_commands! {
///    "add_player" => add_player,
/// };
///
/// fn add_player(In((name, age)): In<(String, f32)>) {
///     println!("Adding player {name} with age {age}");
/// }
///```
#[macro_export]
macro_rules! yarn_commands {
    ($($name:expr => $function:expr),* $(,)?) => {
        {
            let mut map = YarnCommands::default();
            $(
                map.add_command($name, $function);
            )*
            map
        }
    };
}

use crate::commands::command_registry::wait::Wait;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::tasks::{AsyncComputeTaskPool, Task, TaskPool};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn can_add_fn_with_empty_tuple_in_args() {
        let mut methods = YarnCommands::default();
        methods.add_command("test", |_: In<()>| {});
    }

    #[test]
    fn can_add_fn_with_one_in_arg() {
        let mut methods = YarnCommands::default();
        methods.add_command("test", |_: In<f32>| {});
    }

    #[test]
    #[should_panic = "It works!"]
    fn can_call_fn_with_no_args() {
        let mut methods = YarnCommands::default();

        #[allow(clippy::unused_unit)] // Needed for 2024 edition
        methods.add_command("test", |_: In<()>| -> () { panic!("It works!") });
        let method = methods.get_mut("test").unwrap();
        let mut app = App::new();
        method.call(vec![], app.world_mut());
    }

    #[test]
    fn can_call_fn_with_one_arg() {
        let mut methods = YarnCommands::default();

        methods.add_command("test", |In(a): In<f32>| assert_eq!(1.0, a));
        let method = methods.get_mut("test").unwrap();
        let mut app = App::new();
        method.call(to_method_params([1.0]), app.world_mut());
    }

    #[test]
    fn can_add_multiple_fns() {
        let mut methods = YarnCommands::default();

        methods.add_command("test1", |_: In<()>| {});
        methods.add_command("test2", |_: In<f32>| {});
    }

    #[test]
    fn can_call_multiple_fns() {
        let mut methods = YarnCommands::default();
        methods.add_command("test1", |_: In<()>| {});
        methods.add_command("test2", |In(a): In<f32>| assert_eq!(1.0, a));

        let mut app = App::new();
        {
            let method1 = methods.get_mut("test1").unwrap();
            method1.call(vec![], app.world_mut());
        }
        let method2 = methods.get_mut("test2").unwrap();
        method2.call(to_method_params([1.0]), app.world_mut());
    }

    #[test]
    fn can_mutate_world() {
        let mut methods = YarnCommands::default();
        methods.add_command("test", |In(a): In<f32>, mut commands: Commands| {
            commands.insert_resource(Data(a))
        });

        #[derive(Resource)]
        struct Data(f32);

        let method = methods.get_mut("test").unwrap();

        let mut app = App::new();
        method.call(to_method_params([1.0]), app.world_mut());
        let data = app.world().resource::<Data>();
        assert_eq!(data.0, 1.0);
    }

    fn to_method_params(params: impl IntoIterator<Item = impl Into<YarnValue>>) -> Vec<YarnValue> {
        params.into_iter().map(Into::into).collect()
    }

    #[test]
    fn executes_task() {
        let mut methods = YarnCommands::default();
        methods.add_command("test", |_: In<()>| -> Task<()> {
            let thread_pool = AsyncComputeTaskPool::get_or_init(TaskPool::new);
            thread_pool.spawn(async move { sleep(Duration::from_millis(500)) })
        });
        let method = methods.get_mut("test").unwrap();

        let mut app = App::new();
        let task = method.call(vec![], app.world_mut());
        assert!(!task.is_finished());
        sleep(Duration::from_millis(600));
        assert!(task.is_finished());
    }

    #[test]
    fn debug_prints_signature() {
        let mut methods = YarnCommands::default();

        methods.add_command("test", |_: In<(f32, f32)>| {});
        let debug_string = format!("{:?}", methods);

        let element_start = debug_string.find('{').unwrap();
        // This looks like an off-by-one error on closer inspection,
        // but on even closer inspection it's correct because there's a space before the second '{' that we don't want to include.
        let element_end = element_start + debug_string[element_start + 1..].find('{').unwrap();
        let element = &debug_string[element_start..element_end];

        // Not testing the part after because its stability is not guaranteed.
        assert_eq!(
            element,
            "{\"test\": fn(bevy_ecs::system::input::In<(f32, f32)>)"
        );
    }
}

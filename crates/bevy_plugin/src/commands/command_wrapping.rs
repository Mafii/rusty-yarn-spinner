use crate::prelude::*;
use bevy::ecs::system::{SystemParam, SystemParamItem, SystemState};
use bevy::prelude::*;
use bevy::tasks::Task;
use bevy::utils::all_tuples;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use yarnspinner::core::{YarnFnParam, YarnFnParamItem, YarnValueWrapper};

pub(crate) fn command_wrapping_plugin(_app: &mut App) {}

/// A method that can be registered as a command for Yarn files via [`YarnCommands::add_command`].
///
/// The signature of the method must adhere to the following rules:
/// The first parameter must be of the type `In<T>`, where `T` can be converted into a [`YarnFnParam`]. This stands for the parameters passed to the command from Yarn.
/// Multiple parameters are supported as values wrapped in a tuple.
/// For example, to register a command that is called from Yarn like `<<add_player "John" 42>>`, the first parameter must be of the type `In<(String, i32)>` and a call to `register_command` might look like this:
/// ```rust
/// # use bevy_yarnspinner::prelude::*;
/// # use bevy::prelude::*;
/// # let mut commands = YarnCommands::new();
/// commands.add_command("add_player", add_player);
///
/// fn add_player(In((name, age)): In<(String, f32)>) {
///     println!("Adding player {name} with age {age}");
/// }
/// ```
///
/// The parameters following the `In` parameter are taken from the Bevy ECS as any other system would. For example, the following command would print the elapsed time since the game started:
/// ```rust
/// # use bevy_yarnspinner::prelude::*;
/// # use bevy::prelude::*;
/// # let mut commands = YarnCommands::new();
/// commands.add_command("add_player", add_player);
///
/// fn add_player(_: In<()>, time: Res<Time>) {
///    println!("Time since game start: {}", time.elapsed_secs());
/// }
/// ```
/// This command can be called from Yarn with `<<print_time>>`. Note how because we accept no parameters from Yarn, we use `In<()>` as the first parameter.
///
/// The return value must be of a type implementing [`TaskFinishedIndicator`], which is generally either `()` or some kind of wrapped boolean
/// or [`Task`]. If you return something else than `()`, the command will be considered finished when the respective [`TaskFinishedIndicator`] says so.
/// Until then, the dialogue will not be advanced when [`DialogueRunner::continue_in_next_update`] is called. This allows you to e.g. move the camera before the dialogue continues.
/// If you return `()`, the command will be considered finished immediately.
pub trait YarnCommand<Marker>: Send + Sync + 'static + Clone {
    /// The input type used to determine the parameters passed to the command from Yarn. A tuple of values will be interpreted as multiple parameters.
    /// This also counts for arbitrarily nested tuples, which will be flattened.
    type In: YarnFnParam;
    /// The return type of the command. If there is no return value, this is `()`, which means the command is considered finished immediately.
    type Out: TaskFinishedIndicator;
    /// The parameters passed to the command from the Bevy ECS.
    type Param: SystemParam;

    #[doc(hidden)]
    fn run(
        &mut self,
        input: YarnFnParamItem<Self::In>,
        param_value: SystemParamItem<Self::Param>,
    ) -> Self::Out;
}

macro_rules! impl_command_function {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<Input, Func: Send + Sync + 'static, Output, $($param: SystemParam),*> YarnCommand<fn(In<Input>, $($param,)*) -> Output> for Func
        where
            Input: YarnFnParam,
            Output: TaskFinishedIndicator,
            Func: Clone,
        for <'a> &'a mut Func:
            FnMut(In<Input>, $($param), *) -> Output +
            FnMut(In<Input>, $(SystemParamItem<$param>),*) -> Output +
            FnMut(In<YarnFnParamItem<Input>>, $($param), *) -> Output +
            FnMut(In<YarnFnParamItem<Input>>, $(SystemParamItem<$param>),*) -> Output
        {
            type In = Input;
            type Out = Output;
            type Param = ($($param,)*);
            #[inline]
            fn run(&mut self, input: YarnFnParamItem<Input>, param_value: SystemParamItem< ($($param,)*)>) -> Self::Out {
                #[allow(clippy::too_many_arguments)]
                fn call_inner<Input: YarnFnParam, Output: TaskFinishedIndicator, $($param,)*>(
                    mut f: impl FnMut(In<YarnFnParamItem<Input>>, $($param,)*) -> Output,
                    input: In<YarnFnParamItem<Input>>,
                    $($param: $param,)*
                ) -> Output {
                    f(input, $($param,)*)
                }
                let ($($param,)*) = param_value;
                call_inner(self, In(input), $($param),*)
            }
        }
    };
}

// Note that we rely on the highest impl to be <= the highest order of the tuple impls
// of `SystemParam` created.
all_tuples!(impl_command_function, 0, 16, F);

/// Trait implemented by the return types of methods registered in the [`YarnCommands`].
pub trait TaskFinishedIndicator: Debug + Send + Sync + 'static {
    /// Returns `true` if the task is finished.
    fn is_finished(&self) -> bool;
}

impl TaskFinishedIndicator for AtomicBool {
    fn is_finished(&self) -> bool {
        self.load(Ordering::Relaxed)
    }
}

impl TaskFinishedIndicator for bool {
    fn is_finished(&self) -> bool {
        *self
    }
}

impl<T: TaskFinishedIndicator> TaskFinishedIndicator for Arc<T> {
    fn is_finished(&self) -> bool {
        T::is_finished(self.as_ref())
    }
}

impl<T: TaskFinishedIndicator> TaskFinishedIndicator for RwLock<T> {
    fn is_finished(&self) -> bool {
        self.read().unwrap().is_finished()
    }
}

impl<T: TaskFinishedIndicator> TaskFinishedIndicator for Vec<T> {
    fn is_finished(&self) -> bool {
        self.iter().all(|t| t.is_finished())
    }
}

impl TaskFinishedIndicator for Task<()> {
    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}

macro_rules! impl_task_finished_indicator {
    ($($param: ident),*) => {
        impl<$($param: TaskFinishedIndicator),*> TaskFinishedIndicator for ($($param,)*) where ($($param,)*): Debug {
            #[allow(non_snake_case)]
            fn is_finished(&self) -> bool {
                let ($($param,)*) = self;
                $($param.is_finished() &&)* true
            }
        }
    };
}
all_tuples!(impl_task_finished_indicator, 0, 16, F);

/// A type-erased [`YarnCommand`] as it appears in the [`YarnCommands`].
pub trait UntypedYarnCommand: Debug + Send + Sync + 'static {
    #[doc(hidden)]
    fn call(&mut self, input: Vec<YarnValue>, world: &mut World) -> Box<dyn TaskFinishedIndicator>;
    #[doc(hidden)]
    fn clone_box(&self) -> Box<dyn UntypedYarnCommand>;
}

impl Clone for Box<dyn UntypedYarnCommand> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<T, Marker> UntypedYarnCommand for YarnCommandWrapper<Marker, T>
where
    Marker: 'static,
    T: YarnCommand<Marker>,
{
    fn call(&mut self, input: Vec<YarnValue>, world: &mut World) -> Box<dyn TaskFinishedIndicator> {
        let mut system_state: SystemState<T::Param> = SystemState::new(world);
        let param = system_state.get_mut(world);
        let mut input: Vec<_> = input.into_iter().map(YarnValueWrapper::from).collect();
        let mut iter = input.iter_mut().peekable();
        let input = T::In::retrieve(&mut iter);
        assert!(
            iter.next().is_none(),
            "Passed too many arguments to Command"
        );
        let task = YarnCommand::run(&mut self.function, input, param);
        system_state.apply(world);
        Box::new(task)
    }

    fn clone_box(&self) -> Box<dyn UntypedYarnCommand> {
        Box::new(self.clone())
    }
}

pub(crate) struct YarnCommandWrapper<Marker, F>
where
    F: YarnCommand<Marker>,
{
    function: F,

    // NOTE: PhantomData<fn()-> T> gives this safe Send/Sync impls
    _marker: PhantomData<fn() -> Marker>,
}

impl<Marker, F> Clone for YarnCommandWrapper<Marker, F>
where
    F: YarnCommand<Marker>,
{
    fn clone(&self) -> Self {
        Self {
            function: self.function.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Marker, F> From<F> for YarnCommandWrapper<Marker, F>
where
    F: YarnCommand<Marker>,
{
    fn from(function: F) -> Self {
        Self {
            function,
            _marker: PhantomData,
        }
    }
}

impl<Marker, F> Debug for YarnCommandWrapper<Marker, F>
where
    F: YarnCommand<Marker>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let signature = std::any::type_name::<Marker>();
        let function_path = std::any::type_name::<F>();
        let debug_message = format!("{signature} {{{function_path}}}");
        f.debug_struct(&debug_message).finish()
    }
}

impl<Marker, F> Display for YarnCommandWrapper<Marker, F>
where
    F: YarnCommand<Marker>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let signature = std::any::type_name::<Marker>();
        f.write_str(signature)
    }
}

impl PartialEq for Box<dyn UntypedYarnCommand> {
    fn eq(&self, other: &Self) -> bool {
        // Not guaranteed to be unique, but it's good enough for our purposes.
        let debug = format!("{:?}", self);
        let other_debug = format!("{:?}", other);
        debug == other_debug
    }
}

impl Eq for Box<dyn UntypedYarnCommand> {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use bevy::tasks::AsyncComputeTaskPool;

    #[test]
    #[ignore]
    fn accepts_empty_function() {
        fn _f() {}
        // Currently not supported
        // accepts_yarn_command(f);
    }

    #[test]
    fn accepts_empty_tuple_in_param() {
        fn f(_: In<()>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_function_with_simple_in_param() {
        fn f(_: In<usize>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_function_with_optional_in_param() {
        fn f(_: In<Option<usize>>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_function_with_tuple_in_param() {
        fn f(_: In<(usize, isize, (String, &str))>) {}
        accepts_yarn_command(f);
    }

    #[test]
    #[ignore]
    fn accepts_only_query() {
        fn _f(_: Query<Entity>) {}
        // Currently not supported
        // accepts_yarn_command(f);
    }

    #[test]
    fn accepts_empty_tuple_in_param_and_query() {
        fn f(_: In<()>, _: Query<Entity>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_function_with_simple_in_param_and_query() {
        fn f(_: In<usize>, _: Query<Entity>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_function_with_tuple_in_param_and_query() {
        fn f(_: In<(usize, isize, (String, &str))>, _: Query<Entity>) {}
        accepts_yarn_command(f);
    }

    #[test]
    fn accepts_returning_task() {
        fn f(_: In<()>) -> Task<()> {
            let thread_pool = AsyncComputeTaskPool::get();
            thread_pool.spawn(async move {
                println!("Hello from task");
            })
        }
        accepts_yarn_command(f);
    }

    macro_rules! assert_is_yarn_command {
        (($($param:ty),*) -> $ret:ty) => {
            static_assertions::assert_impl_all!(fn($($param),*) -> $ret: YarnCommand<fn($($param),*) -> $ret>);
        };
    }

    macro_rules! assert_is_not_yarn_command {
        (($($param:ty),*) -> $ret:ty) => {
            static_assertions::assert_not_impl_any!(fn($($param),*) -> $ret: YarnCommand<fn($($param),*) -> $ret>);
        };
    }

    assert_is_yarn_command! { (In<((), Option<()>)>) -> bool }
    assert_is_not_yarn_command! { (In<(Option<()>, ())>) -> bool }

    fn accepts_yarn_command<Marker>(_: impl YarnCommand<Marker>) {}
}

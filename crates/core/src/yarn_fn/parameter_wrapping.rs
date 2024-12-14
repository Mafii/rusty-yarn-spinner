//! This helper code allows us to pass params to YarnFns by value (e.g. `usize`), by reference (e.g. (`&usize`) or by [`std::borrow::Borrow`] (e.g. `String` -> `&str`)
//!
//! Inspired by <https://promethia-27.github.io/dependency_injection_like_bevy_from_scratch/chapter2/passing_references.html>

use super::optionality::{AllowedOptionalityChain, Optional, Optionality, Required};
use crate::prelude::*;
use std::any::Any;
use std::borrow::Borrow;
use std::fmt::{Debug, Display};
use std::iter::Peekable;
use std::marker::PhantomData;
use std::slice::IterMut;
use yarnspinner_macros::all_tuples;

/// Helper class for implementing something like [`YarnFn`] yourself.
/// You probably don't want to use this directly as a consumer unless you're doing some wizardry.
#[derive(Debug)]
pub struct YarnValueWrapper {
    raw: Option<YarnValue>,
    converted: Option<Box<dyn Any>>,
}

#[doc(hidden)]
pub type YarnValueWrapperIter<'a> = Peekable<IterMut<'a, YarnValueWrapper>>;

impl From<YarnValue> for YarnValueWrapper {
    fn from(value: YarnValue) -> Self {
        Self {
            raw: Some(value),
            converted: None,
        }
    }
}

impl YarnValueWrapper {
    fn convert<T>(&mut self)
    where
        T: TryFrom<YarnValue> + 'static,
        <T as TryFrom<YarnValue>>::Error: Display,
    {
        let raw = std::mem::take(&mut self.raw).unwrap();
        let converted: T = raw
            .try_into()
            .unwrap_or_else(|e| panic!("Parameter passed to Yarn has invalid type: {e}"));
        self.converted.replace(Box::new(converted));
    }
}

/// Trait implemented by types that can be used in [`YarnFn`]-like contexts. Implemented by the following types and references of them:
/// - [`bool`]
/// - Numeric type, i.e. one of [`f32`], [`f64`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`isize`]
/// - [`String`] (for a reference, [`&str`] may be used instead of `&String`)
/// - [`YarnValue`], which means that a parameter may be any of the above types
/// - Tuples of the above types.
pub trait YarnFnParam {
    /// The item type returned when constructing this [`YarnFn`] param. The value of this associated type should be `Self`, instantiated with a new lifetime.
    /// You could think of `YarnFnParam::Item<'new>` as being an operation that changes the lifetime bound to `Self`.
    type Item<'new>;

    /// Tracks if this parameter is optional or required.
    /// This information is used to disallow required parameters to follow optional ones.
    /// See the [`AllowedOptionalityChain`] trait for details.
    #[doc(hidden)]
    type Optionality: Optionality;

    #[doc(hidden)]
    fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a>;
}

/// Shorthand way of accessing the associated type [`YarnFnParam::Item`] for a given [`YarnFnParam`].
pub type YarnFnParamItem<'a, P> = <P as YarnFnParam>::Item<'a>;

impl<T: YarnFnParam> YarnFnParam for Option<T> {
    type Item<'new> = Option<T::Item<'new>>;
    type Optionality = Optional;

    fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
        if iter.peek().is_some() {
            Some(T::retrieve(iter))
        } else {
            None
        }
    }
}

macro_rules! impl_yarn_fn_param_tuple {
    ($($param: ident),*) => {
        #[allow(non_snake_case)]
        impl<$($param,)*> YarnFnParam for ($($param,)*)
            where $($param: YarnFnParam,)*
                  ($(<$param as YarnFnParam>::Optionality,)*): AllowedOptionalityChain
        {
            type Item<'new> = ($($param::Item<'new>,)*);
            type Optionality = <($(<$param as YarnFnParam>::Optionality,)*) as AllowedOptionalityChain>::Last;

            #[allow(unused_variables, clippy::unused_unit)] // for n = 0 tuples
            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
               ($($param::retrieve(iter),)*)
            }
        }
    };
}

all_tuples!(impl_yarn_fn_param_tuple, 0, 16, P);

struct ResRef<'a, T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
{
    value: &'a T,
    phantom_data: PhantomData<T>,
}

impl<T> YarnFnParam for ResRef<'_, T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
{
    type Item<'new> = ResRef<'new, T>;
    type Optionality = Required;

    fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
        let value = iter.next().expect("Passed too few arguments to YarnFn");
        value.convert::<T>();
        let converted = value.converted.as_ref().unwrap();
        let value = converted.downcast_ref::<T>().unwrap();
        ResRef {
            value,
            phantom_data: PhantomData,
        }
    }
}

/// For types like `String`, of which a reference to `&str` and not `&String`.
/// These kinds of types implement [`Borrow`], hence this struct's name
struct ResRefBorrow<'a, T, U>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
    T: Borrow<U>,
    U: ?Sized + 'static,
{
    value: &'a U,
    phantom_data: PhantomData<T>,
}

impl<T, U> YarnFnParam for ResRefBorrow<'_, T, U>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
    T: Borrow<U>,
    U: ?Sized + 'static,
{
    type Item<'new> = ResRefBorrow<'new, T, U>;
    type Optionality = Required;

    fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
        let value = iter.next().expect("Passed too few arguments to YarnFn");
        value.convert::<T>();
        let converted = value.converted.as_ref().unwrap();
        let value = converted.downcast_ref::<T>().unwrap();
        ResRefBorrow {
            value: value.borrow(),
            phantom_data: PhantomData,
        }
    }
}

struct ResOwned<T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
{
    value: T,
}

impl<T> YarnFnParam for ResOwned<T>
where
    T: TryFrom<YarnValue> + 'static,
    <T as TryFrom<YarnValue>>::Error: Display,
{
    type Item<'new> = ResOwned<T>;
    type Optionality = Required;

    fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
        let value = iter.next().expect("Passed too few arguments to YarnFn");
        value.convert::<T>();
        let converted = value.converted.take().unwrap();
        let value = *converted.downcast::<T>().unwrap();
        ResOwned { value }
    }
}

macro_rules! impl_yarn_fn_param {
    ([$($referenced:ty $(=> $owned:ty)?),*]: YarnFnParam) => {
        $(
            impl_yarn_fn_param_inner!{
                $referenced $(=> $owned)?: YarnFnParam
            }
        )*
    }
}

macro_rules! impl_yarn_fn_param_inner {
    ($referenced:ty: YarnFnParam) => {
        impl YarnFnParam for &$referenced {
            type Item<'new> = &'new $referenced;
            type Optionality = Required;

            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
                ResRef::<$referenced>::retrieve(iter).value
            }
        }

        impl YarnFnParam for $referenced {
            type Item<'new> = $referenced;
            type Optionality = Required;

            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
                ResOwned::<$referenced>::retrieve(iter).value
            }
        }
    };
    ($referenced:ty => $owned:ty: YarnFnParam) => {
        impl YarnFnParam for &$referenced {
            type Item<'new> = &'new $referenced;
            type Optionality = Required;

            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
                ResRefBorrow::<$owned, $referenced>::retrieve(iter).value
            }
        }

        impl YarnFnParam for &$owned {
            type Item<'new> = &'new $owned;
            type Optionality = Required;

            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
                ResRef::<$owned>::retrieve(iter).value
            }
        }

        impl YarnFnParam for $owned {
            type Item<'new> = $owned;
            type Optionality = Required;

            fn retrieve<'a>(iter: &mut YarnValueWrapperIter<'a>) -> Self::Item<'a> {
                ResOwned::<$owned>::retrieve(iter).value
            }
        }
    };
}

impl_yarn_fn_param! {
    [str => String, YarnValue, bool, f32, f64, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, isize]: YarnFnParam
}

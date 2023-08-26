use leptos::html::ElementDescriptor;
use leptos::*;
use std::marker::PhantomData;
use std::ops::Deref;

/// Used as an argument type to make it easily possible to pass either
/// * a `web_sys` element that implements `E` (for example `EventTarget` or `Element`),
/// * an `Option<T>` where `T` is the web_sys element,
/// * a `Signal<T>` where `T` is the web_sys element,
/// * a `Signal<Option<T>>` where `T` is the web_sys element,
/// * a `NodeRef`
/// into a function. Used for example in [`use_event_listener`].
pub enum ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    Static(Option<T>),
    Dynamic(Signal<Option<T>>),
    _Phantom(PhantomData<E>),
}

impl<T, E> Default for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    fn default() -> Self {
        Self::Static(None)
    }
}

impl<T, E> Clone for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::Static(t) => Self::Static(t.clone()),
            Self::Dynamic(s) => Self::Dynamic(*s),
            _ => unreachable!(),
        }
    }
}

impl<T, E> SignalGet for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    type Value = Option<T>;
    fn get(&self) -> Option<T> {
        match self {
            Self::Static(t) => t.clone(),
            Self::Dynamic(s) => s.get(),
            _ => unreachable!(),
        }
    }

    fn try_get(&self) -> Option<Option<T>> {
        match self {
            Self::Static(t) => Some(t.clone()),
            Self::Dynamic(s) => s.try_get(),
            _ => unreachable!(),
        }
    }
}

impl<T, E> SignalWith for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    type Value = Option<T>;
    fn with<O>(&self, f: impl FnOnce(&Option<T>) -> O) -> O {
        match self {
            Self::Static(t) => f(t),
            Self::Dynamic(s) => s.with(f),
            _ => unreachable!(),
        }
    }

    fn try_with<O>(&self, f: impl FnOnce(&Option<T>) -> O) -> Option<O> {
        match self {
            Self::Static(t) => Some(f(t)),
            Self::Dynamic(s) => s.try_with(f),
            _ => unreachable!(),
        }
    }
}

impl<T, E> SignalWithUntracked for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    type Value = Option<T>;
    fn with_untracked<O>(&self, f: impl FnOnce(&Option<T>) -> O) -> O {
        match self {
            Self::Static(t) => f(t),
            Self::Dynamic(s) => s.with_untracked(f),
            _ => unreachable!(),
        }
    }

    fn try_with_untracked<O>(&self, f: impl FnOnce(&Option<T>) -> O) -> Option<O> {
        match self {
            Self::Static(t) => Some(f(t)),
            Self::Dynamic(s) => s.try_with_untracked(f),
            _ => unreachable!(),
        }
    }
}

impl<T, E> SignalGetUntracked for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    type Value = Option<T>;
    fn get_untracked(&self) -> Option<T> {
        match self {
            Self::Static(t) => t.clone(),
            Self::Dynamic(s) => s.get_untracked(),
            _ => unreachable!(),
        }
    }

    fn try_get_untracked(&self) -> Option<Option<T>> {
        match self {
            Self::Static(t) => Some(t.clone()),
            Self::Dynamic(s) => s.try_get_untracked(),
            _ => unreachable!(),
        }
    }
}

// From static element //////////////////////////////////////////////////////////////

impl<T, E> From<T> for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    fn from(value: T) -> Self {
        ElementMaybeSignal::Static(Some(value))
    }
}

impl<T, E> From<Option<T>> for ElementMaybeSignal<T, E>
where
    T: Into<E> + Clone + 'static,
{
    fn from(target: Option<T>) -> Self {
        ElementMaybeSignal::Static(target)
    }
}

// From string (selector) ///////////////////////////////////////////////////////////////

impl<'a, E> From<&'a str> for ElementMaybeSignal<web_sys::Element, E>
where
    E: From<web_sys::Element> + 'static,
{
    fn from(target: &'a str) -> Self {
        Self::Static(document().query_selector(target).unwrap_or_default())
    }
}

impl<E> From<String> for ElementMaybeSignal<web_sys::Element, E>
where
    E: From<web_sys::Element> + 'static,
{
    fn from(target: String) -> Self {
        Self::Static(document().query_selector(&target).unwrap_or_default())
    }
}

impl<E> From<Signal<String>> for ElementMaybeSignal<web_sys::Element, E>
where
    E: From<web_sys::Element> + 'static,
{
    fn from(signal: Signal<String>) -> Self {
        Self::Dynamic(
            create_memo(move |_| document().query_selector(&signal.get()).unwrap_or_default())
                .into(),
        )
    }
}

// From signal ///////////////////////////////////////////////////////////////

macro_rules! impl_from_signal_option {
    ($ty:ty) => {
        impl<T, E> From<$ty> for ElementMaybeSignal<T, E>
        where
            T: Into<E> + Clone + 'static,
        {
            fn from(target: $ty) -> Self {
                Self::Dynamic(target.into())
            }
        }
    };
}

impl_from_signal_option!(Signal<Option<T>>);
impl_from_signal_option!(ReadSignal<Option<T>>);
impl_from_signal_option!(RwSignal<Option<T>>);
impl_from_signal_option!(Memo<Option<T>>);

macro_rules! impl_from_signal {
    ($ty:ty) => {
        impl<T, E> From<$ty> for ElementMaybeSignal<T, E>
        where
            T: Into<E> + Clone + 'static,
        {
            fn from(signal: $ty) -> Self {
                Self::Dynamic(Signal::derive(move || Some(signal.get())))
            }
        }
    };
}

impl_from_signal!(Signal<T>);
impl_from_signal!(ReadSignal<T>);
impl_from_signal!(RwSignal<T>);
impl_from_signal!(Memo<T>);

// From NodeRef //////////////////////////////////////////////////////////////

macro_rules! impl_from_node_ref {
    ($ty:ty) => {
        impl<R> From<NodeRef<R>> for ElementMaybeSignal<$ty, $ty>
        where
            R: ElementDescriptor + Clone + 'static,
        {
            fn from(node_ref: NodeRef<R>) -> Self {
                Self::Dynamic(Signal::derive(move || {
                    node_ref.get().map(move |el| {
                        let el = el.into_any();
                        let el: $ty = el.deref().clone().into();
                        el
                    })
                }))
            }
        }
    };
}

impl_from_node_ref!(web_sys::EventTarget);
impl_from_node_ref!(web_sys::Element);

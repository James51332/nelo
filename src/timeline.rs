use crate::signal::Signal;
use std::sync::Arc;

/// Timeline is an instance of a signal or a constant.
pub enum Timeline<T> {
    Constant(T),
    Dynamic(Arc<dyn Signal<Output = T>>),
}

impl<T: 'static> Timeline<T> {
    pub fn constant(s: T) -> Self {
        Self::Constant(s)
    }

    pub fn dynamic(s: impl Signal<Output = T> + 'static) -> Self {
        Self::Dynamic(Arc::new(s))
    }

    pub fn length(&self) -> Option<f64> {
        match self {
            Self::Constant(_) => None,
            Self::Dynamic(s) => s.length(),
        }
    }
}

impl<T: Clone + 'static> Timeline<T> {
    pub fn sample(&self, t: f64) -> T {
        match self {
            Self::Constant(s) => s.clone(),
            Self::Dynamic(s) => s.sample(t),
        }
    }

    fn into_signal(self) -> Arc<dyn Signal<Output = T>> {
        match self {
            Self::Constant(s) => Arc::new(Const(s)),
            Self::Dynamic(s) => s,
        }
    }

    pub fn with_length(self, length: f64) -> Timeline<T> {
        Self::Dynamic(Arc::new(WithLength {
            inner: self.into_signal(),
            length,
        }))
    }
}

// Helpers to allow us to add a length to any existing timeline.
struct Const<T>(T);
impl<T: Clone + 'static> Signal for Const<T> {
    type Output = T;
    fn sample(&self, _t: f64) -> Self::Output {
        self.0.clone()
    }
}

pub struct WithLength<T: 'static> {
    inner: Arc<dyn Signal<Output = T>>,
    length: f64,
}

impl<T: 'static> Signal for WithLength<T> {
    type Output = T;
    fn sample(&self, t: f64) -> Self::Output {
        self.inner.sample(t)
    }
    fn length(&self) -> Option<f64> {
        Some(self.length)
    }
}

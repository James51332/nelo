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
}

impl<T: Clone + 'static> Timeline<T> {
    pub fn sample(&self, t: f64) -> T {
        match self {
            Self::Constant(s) => s.clone(),
            Self::Dynamic(s) => s.sample(t),
        }
    }
}

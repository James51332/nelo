//! Signals  arevalues that vary over time.

/// A value that varies over time.
pub trait Signal: 'static {
    /// The value produced at each instant.
    type Output;

    /// Sample the value at time `t`, in seconds. Will not panic if sampled out
    /// of bounds.
    fn sample(&self, t: f32) -> Self::Output;

    /// The signal's finite duration in seconds, or `None` if it runs forever.
    fn length(&self) -> Option<f32> {
        None
    }
}

/// Any closure `Fn(f32) -> T + 'static` is a signal, with no length.
impl<T: 'static, F: Fn(f32) -> T + Clone + 'static> Signal for F {
    type Output = T;

    fn sample(&self, t: f32) -> T {
        self(t)
    }
}

pub trait SignalClone: Signal + 'static {
    /// We require that Signals are cloneable into a box.
    fn clone_box(&self) -> Box<dyn SignalClone<Output = Self::Output> + 'static>;
}

impl<T: Signal + Clone> SignalClone for T {
    fn clone_box(&self) -> Box<dyn SignalClone<Output = Self::Output> + 'static> {
        Box::new(self.clone())
    }
}

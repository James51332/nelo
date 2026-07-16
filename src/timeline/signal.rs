//! Signals: values that vary over time.
//!
//! A [`Signal`] is the pure, declarative half of animation — a function from a
//! time `t` (in seconds) to a value. It owns no GPU state and draws nothing; it
//! only *describes* how something evolves. The renderer samples signals each
//! frame to turn that description into the concrete numbers it draws.
//!
//! Because any `Fn(f32) -> T` is already a signal (see the blanket impl below),
//! the simplest signal is a plain closure — `|t| t.sin()` — and richer ones are
//! built by composing rather than by writing a new type for each effect.

/// A value that varies over time.
///
/// Implementors map a time `t` (seconds, increasing as playback advances) to an
/// [`Output`](Signal::Output). Sampling should be *pure*: the same `t` must
/// always yield the same value. That is what lets a frame be rendered live,
/// re-rendered on resize, or exported out of order and still look identical.
///
/// The `'static` bound lets a signal be type-erased into an `Arc<dyn Signal>`
/// and shared — see [`Timeline`](crate::timeline::Timeline).
pub trait Signal: 'static {
    /// The value produced at each instant.
    type Output;

    /// Sample the value at time `t`, in seconds.
    fn sample(&self, t: f32) -> Self::Output;

    /// The signal's finite duration in seconds, or `None` if it runs forever.
    ///
    /// Advisory only — sampling past the length is still valid — but it lets a
    /// scheduler know when a signal has "ended". Defaults to unbounded.
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

//! Signals: values that vary over time.
//!
//! A [`Signal`] is the pure, declarative half of animation — a function from a
//! time `t` (in seconds) to a value. It owns no GPU state and draws nothing; it
//! only *describes* how something evolves. The renderer samples signals each
//! frame to turn that description into the concrete numbers it draws.
//!
//! Because any `Fn(f64) -> T` is already a signal (see the blanket impl below),
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
    fn sample(&self, t: f64) -> Self::Output;

    /// The signal's finite duration in seconds, or `None` if it runs forever.
    ///
    /// Advisory only — sampling past the length is still valid — but it lets a
    /// scheduler know when a signal has "ended". Defaults to unbounded.
    fn length(&self) -> Option<f64> {
        None
    }
}

/// Any closure `Fn(f64) -> T` is a signal, so no wrapper type is needed to lift
/// ordinary functions into the animation system. A closure carries no duration,
/// so [`length`](Signal::length) stays `None`; attach one with
/// [`Timeline::with_length`](crate::timeline::Timeline::with_length).
impl<T: 'static, F: Fn(f64) -> T + 'static> Signal for F {
    type Output = T;
    fn sample(&self, t: f64) -> T {
        self(t)
    }
}

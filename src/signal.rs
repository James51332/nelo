/// Any type which implements this trait can become a timeline.
pub trait Signal: 'static {
    type Output;
    fn sample(&self, t: f64) -> Self::Output;
    fn length(&self) -> Option<f64> {
        None
    }
}

impl<T: 'static, F: Fn(f64) -> T + 'static> Signal for F {
    type Output = T;
    fn sample(&self, t: f64) -> T {
        self(t)
    }
}

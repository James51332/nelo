pub trait Signal: 'static {
    type Output;
    fn sample(&self, t: f64) -> Self::Output;
}

// Closures can certainly be considered signals.
impl<T: 'static, F: Fn(f64) -> T + 'static> Signal for F {
    type Output = T;
    fn sample(&self, t: f64) -> T {
        self(t)
    }
}

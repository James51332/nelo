//! Render simple lines as a grid.

use crate::scene::{GroupRef, Path, Scene};
use glam::Vec2;

impl Scene {
    /// Creates x and y axes over interval (-10, 10)
    pub fn axes(&mut self) -> GroupRef<'_> {
        self.axes_with_count((21, 21))
    }

    /// Creates x and y lines in a group using spacing of 1 according to the count.
    pub fn axes_with_count(&mut self, count: (u32, u32)) -> GroupRef<'_> {
        // Compute the bounds of the axes.
        let x_steps = count.0;
        let x_max = (x_steps as f32 - 1.0) / 2.0;
        let x_min = -x_max;

        let y_steps = count.1;
        let y_max = (y_steps as f32 - 1.0) / 2.0;
        let y_min = -y_max;

        // Create horizontal lines first, then vertical.
        self.group()
            .create(y_steps, |i, s| {
                let y = y_min + i as f32;
                let line = Path::line(Vec2::new(x_min, y), Vec2::new(x_max, y));
                s.spline(line)
            })
            .create(x_steps, |i, s| {
                let x = x_min + i as f32;
                let line = Path::line(Vec2::new(x, y_min), Vec2::new(x, y_max));
                s.spline(line)
            })
    }
}

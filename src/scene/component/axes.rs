//! Render simple lines as a grid.

use crate::{
    render::Color,
    scene::{GroupRef, Path, Scene},
};
use glam::Vec2;

impl Scene {
    /// Creates x and y axes over x-interval (-8, 8) and y-interval (-5, 5)
    pub fn axes(&mut self) -> GroupRef<'_> {
        self.axes_with_count((17, 9))
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

        // Horizontal lines are one group and vertical are another.
        let mut group = self.group();
        group
            .create(y_steps, |i, s| {
                let y = y_min + i as f32;
                let line = Path::line(Vec2::new(x_min - 1.0, y), Vec2::new(x_max + 1.0, y));
                s.spline(line)
                    .stroke(if y.abs() < 0.001 {
                        Color::WHITE.with_alpha(0.5)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    })
                    .stroke_weight(0.005)
            })
            .create(x_steps, |i, s| {
                let x = x_min + i as f32;
                let line = Path::line(Vec2::new(x, y_min - 1.0), Vec2::new(x, y_max + 1.0));
                s.spline(line)
                    .stroke(if x.abs() < 0.001 {
                        Color::WHITE.with_alpha(0.5)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    })
                    .stroke_weight(0.005)
            });

        group
    }
}

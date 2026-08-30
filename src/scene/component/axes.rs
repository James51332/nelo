//! Render simple lines as a grid.

use crate::{
    scene::{Color, GroupRef, Scene},
    timeline::Path,
};
use glam::Vec2;

impl Scene {
    /// Creates x and y axes over height of the screen at time zero.
    pub fn axes(&mut self) -> GroupRef<'_> {
        // Get the width and height at time zero.
        let height = self.camera.height.sample(0.0);
        let width = self.camera.aspect() * height;

        // Ceil to make sure that we have sufficient length.
        let mut x_lines = width.ceil() as u32;
        let mut y_lines = height.ceil() as u32;

        // Pad to an odd number.
        x_lines += 1 - (x_lines % 2);
        y_lines += 1 - (y_lines % 2);

        self.axes_with_count((x_lines, y_lines))
    }

    /// Creates x and y lines in a group using spacing of 1 according to the count.
    pub fn axes_with_count(&mut self, count: (u32, u32)) -> GroupRef<'_> {
        // Compute the bounds of the axes.
        let x_steps = count.0 as i32;
        let x_max = (x_steps + 1) / 2;
        let x_min = -x_steps / 2;

        let y_steps = count.1 as i32;
        let y_max = (y_steps + 1) / 2;
        let y_min = -y_steps / 2;

        // Horizontal lines are one group and vertical are another.
        let mut group = self.group();
        group
            .create(y_steps as u32, |i, s| {
                let y = y_min as f32 + i as f32;
                let line = Path::line(
                    Vec2::new(x_min as f32 - 1.0, y),
                    Vec2::new(x_max as f32 + 1.0, y),
                );
                s.spline(line)
                    .stroke(if y.abs() < 0.001 {
                        Color::WHITE.with_alpha(0.5)
                    } else {
                        Color::WHITE.with_alpha(0.1)
                    })
                    .stroke_weight(0.005)
            })
            .create(x_steps as u32, |i, s| {
                let x = x_min as f32 + i as f32;
                let line = Path::line(
                    Vec2::new(x, y_min as f32 - 1.0),
                    Vec2::new(x, y_max as f32 + 1.0),
                );
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

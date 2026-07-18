//! A collection of components which are used by the renderers.

use crate::timeline::Timeline;
use glam::prelude::*;

/// A circle has no attached data. It has a radius of 1 but can
/// be scaled using a transform.
pub struct Circle;
pub struct Fill(pub Timeline<Vec4>);

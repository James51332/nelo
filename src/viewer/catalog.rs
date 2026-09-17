//! A source is a playback generator, which can be refreshed.

use crate::scene::Playback;

/// A catalog of playbacks.
pub trait Catalog {
    fn names(&self) -> Vec<String>;

    fn playback(&self, name: &String) -> Option<Playback>;

    fn stale(&self) -> bool {
        false
    }
}

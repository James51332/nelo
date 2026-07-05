# nelo

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

> Moments in a scene are independent of one another. Everything is a function of time.*

At the heart of nelo is the `Timeline`, a generic type which can be queried at any given time.

Timelines can be built from a few sources:
* Constants
* Closures
* Keyframes
* Input (UI, Keyboard, Mouse)

Timelines can also be combined in a few ways:
* Addition - Layer one timeline on another (w/ optional offset)
* Multiplication - Apply first then second transform (e.g. matrix multiplication)
* Composition - Use one timelines output as the time input for another
* Sequencing - Join timelines together in series
* Sampling - Sample one timeline within a closure

On it's own, this API forces robust animations, but semantic sugar elevates the experience
to the level of any iterative simulation tool:
* Convenient Animations
* Grouping & Positioning
* Object Transformations

## to do

* [ ] Timeline
    * [x] Signal Trait
    * [x] Constants
    * [x] Closures
    * [x] Keyframes
    * [ ] Composition
    * [x] Lengths
* [ ] Scene
    * [ ] Entities
    * [ ] Renderer System
    * [ ] Grouping
* [ ] Renderers
    * [x] Circle Renderer
    * [ ] Path Renderer
    * [ ] Text Renderer
    * [ ] Camera Timeline
* [ ] Input Timelines
* [ ] Animations
* [ ] Exporting (via `ffmpeg`)


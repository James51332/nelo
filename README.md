# nelo

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

nelo treats every animation as a pure function of time. No object state, no imperative updates. Just describe what the scene should look like at any time `t`.

* Stateless by design
* Fully scrub-friendly
* Composable: animations don’t interfere
* Ideal for teaching, math, CS, and design

## example

nelo is a very young project, and the timeline API is still being fleshed out. In the future, nelo will primarily use a lua API. However, this will be powered by a robust C++ API. Here is how timelines work so far.

```cpp
// Create a constant timeline. We can add keyframes.
nelo::timeline x = 5.0;
x.add_keyframe(5.0, 24.0, easing::linear)
 .add_keyframe(10, 4, easing::cubicInOut);

// Create a procedural timeline from a lambda.
nelo::timeline y = [](double t) { return t * t; };

// To combine timeline in more complex ways, we can overlay them. 
// e.g. At t=4.0, begin to add the x timeline.
y.add_timeline(4.0, x);

// We sample timelines too.
nelo::log::out("Sampled timeline x at t = 2.0s: {}", x.sample(2.0));
```

Timelines are entirely templated, and here we see that the compiler deduces that these are `timeline<double>`, ANYTHING can be a timeline! Colors, transforms, text, and shapes are all able to be dynamically layered and composed.

## building

nelo uses CMake as a build system, and comes packaged as a static C++ library, a main executable, and a program containing unit tests. To build clone the library and its dependencies, and build with a toolchain of your choosing. **Note: ffmpeg is a required depedencies that is not shipped with nelo.** It can be installed via homebrew if needed.

```
brew install ffmpeg
git clone --recursive https://github.com/james51332/nelo
cd nelo
mkdir build
cd build
cmake ..
cmake --build . --target nelo
```

## doing

There are so many directions to take nelo, but I want to get all of the basics set up for the v0.1 release. Here are the top priorities so far.
1. Simplify Lua API
2. Better OpenGL Buffer (and Framebuffers)
3. Visibility Components
4. Transform Hierarchy/Grouping
5. Improve Curve Rasterization
6. More Shape Renderers 
7. Text/LaTeX Rendering
8. Improve FFmpeg Dependency in Build System
9. Luarocks Installation (w/ Lua autocomplete)

## shaders

Shaders are handled in a powerful manner through CMake in nelo. The shaders directory holds all shaders which come bundled with the library. Whenever these files are changed and the library is built, it automatically pulls them into the engine. The files can be deleted, and the shaders can be loaded all the same. This is fallback, so any changes made to these shaders don't require the engine to be rebuilt. However, do note that the since **the engine will not error if these files are not found; make sure the engine is run with the correct working directory if you modify the shaders without rebuilding**.

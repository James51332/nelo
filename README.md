# nelo

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

nelo treats every animation as a pure function of time. No object state, no imperative updates. Just describe what the scene should look like at any time `t`.

* Stateless by design
* Fully scrub-friendly
* Composable: animations don’t interfere
* Ideal for teaching, math, CS, and design

## overview

nelo is centered around the timeline, which is a completely stateless way that all properties are defined. The timeline is accesible, but interfacing with it is optional. Every scene is composed of entities which have components. Each component is stored a timeline. This means we can for example, add two transform key frames, and ease between them. These properties are also composed of timelines. That way, we can individually apply effects to specific component properties. Here's a sample scene in lua.

```lua
-- Create our scene. The name is video file name. 
local hello_nelo = scene('Hello Nelo')

local object = hello_nelo:create_entity {
  circle = circle {
    fill_color = color(1.0, 1.0, 0.0, 1.0),
    radius = timeline('number', function(t) return 1.5 + 0.5 * math.sin(math.pi * t) end)
  },
  transform = transform {
    position = timeline('vec3', function(t) return vec3(2.0 * math.cos(4.0 * t), 2.0 * math.sin(4.0 * t), 0.0) end)
  }
}

-- Render our scene for 10 seconds.
hello_nelo:play(10.0) -- or hello_nelo:play(0.0, 10.0)
```

## building

nelo uses CMake as a build system, and comes packaged as a static C++ library, a main executable, and a program containing unit tests. To build clone the library and its dependencies, and build with a toolchain of your choosing. **Note: ffmpeg is a required depedencies that is not shipped with nelo.** It can be installed via homebrew if needed. The build system will be revised to support Windows before v0.1 release.

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
1. Better OpenGL Buffer (and Framebuffers)
2. Visibility Components
3. Transform Hierarchy/Grouping
4. Improve Curve Rasterization
5. More Shape Renderers 
6. Text/LaTeX Rendering
7. Improve FFmpeg Dependency in Build System
8. Luarocks Installation (w/ Lua autocomplete)

## shaders

Shaders are handled in a powerful manner through CMake in nelo. The shaders directory holds all shaders which come bundled with the library. Whenever these files are changed and the library is built, it automatically pulls them into the engine. The files can be deleted, and the shaders can be loaded all the same. This is fallback, so any changes made to these shaders don't require the engine to be rebuilt. However, do note that the since **the engine will not error if these files are not found; make sure the engine is run with the correct working directory if you modify the shaders without rebuilding**. The plan in the future is support loading custom shaders, but there is no public API yet.

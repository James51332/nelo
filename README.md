# nelo

A stateless, timeline-driven animation engine for explorable visual animations.

## philosophy

nelo treats every animation as a pure function of time. No object state, no imperative updates. Just describe what the scene should look like at any time `t`.

* Stateless by design
* Fully scrub-friendly
* Composable: animations don’t interfere
* Ideal for teaching, math, CS, and design

## example

nelo is in infancy, and the timeline API is still being fleshed out. In the future, nelo will primarily use a lua API. However, this will be powered by a robust C++ API. Here is how timelines work so far.

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
std::cout << "Sampled timeline x at t = 2.0s: " << x.sample(2.0) << std::endl;
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

These tasks are my top priorities to get a basic prototype for nelo up and running.
1. setup testing via CTest.
 a. setup basic testing system
 b. use Github CI to run the tests automatically.
2. define loop/sequence API for timelines.
3. basic timeline based ECS.
4. shapes renderer via OpenGL.

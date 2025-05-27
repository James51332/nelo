#pragma once

#include <SDL3/SDL.h>

namespace nelo
{

// A context sets up the OpenGL interface via SDL3 and glad. It can be run as a windowed app or
// headless. For now, we'll just give it a default size. In the real pipeline, we'll probably use
// a frame buffer. TODO Investigate using SDL GPU API. This project is fairly new, but may offer a
// very convenient way to run our engine// on other render APIs. Right now it has a questionable
// shader pipeline and lacks support for WebGL. When these are completed, I think // may be worth it
// to implement.
class context
{
public:
  context(bool headless = false);
  ~context();

  // Polls events if running in windowed mode.
  void update();

  // Some simple getters regarding this context.
  bool headless() const { return is_headless; }
  bool active() const { return is_active; }

private:
  // Static field to enforce singleton.
  inline static bool created_singleton = false;

private:
  // We use this even when we are headless.
  SDL_Window* window;

  // Some basic state about this context.
  bool is_headless;
  bool is_active;
};

} // namespace nelo

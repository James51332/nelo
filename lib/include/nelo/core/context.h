#pragma once

#include <SDL3/SDL.h>
#include <cstdint>
#include <glm/glm.hpp>

namespace nelo
{

// A context opens an application with the OS, as well as creates an OpenGL context to render. This
// class has been setup as a singleton, so that it doesn't need to be owned when we load as a lua
// module. When are not headless, we need to poll events, via poll(). Technically, we are leaking
// this class as a lua module, but it's fine as it will only be at termination.
class context
{

public:
  // Creates a context with given width, height, and headless mode.
  static void create(std::uint32_t width = 800, std::uint32_t height = 600, bool headless = true);
  static void kill();

  // Polls events if running in windowed mode.
  static void poll();

  // Presents the back buffer to window. Does nothing in headless mode.
  static void swap();

  // Some simple getters regarding this context.
  static bool headless() { return is_headless; }
  static bool active() { return is_active; }

  // Get the size of the context.
  static std::uint32_t width() { return view_width; }
  static std::uint32_t height() { return view_height; }

private:
  inline static bool initialized = false;

  // We use this even when we are headless.
  inline static SDL_Window* window = nullptr;
  inline static SDL_GLContext render_context;

  inline static std::uint32_t view_width, view_height;
  inline static bool is_headless, is_active = false;
};

} // namespace nelo

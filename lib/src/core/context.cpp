#include "core/context.h"

#include <format>
#include <glad/glad.h>
#include <stdexcept>

#include "core/log.h"

namespace nelo
{

void context::create(std::uint32_t width, std::uint32_t height, bool headless)
{
  if (initialized)
    throw std::runtime_error("Unable to create multiple contexts per app!");
  initialized = true;

  // Set our state according to the given value.
  view_width = width;
  view_height = height;
  is_headless = headless;

  // Initialize SDL library with our proper settings.
  SDL_Init(SDL_INIT_VIDEO);

  // Create our SDL_window with our desired flags.
  SDL_WindowFlags flags = SDL_WINDOW_OPENGL;

  // If we are headless, don't show the window.
  if (is_headless)
    flags |= SDL_WINDOW_HIDDEN;

  // Finally, we can go ahead and create our window.
  window = SDL_CreateWindow("nelo", width, height, flags);

  // If we fail, we can go ahead an throw an error.
  if (!window)
  {
    auto msg = std::format("Unable to create window: {}", SDL_GetError());
    throw std::runtime_error(msg);
  }

  // Set our flags for the OpenGL context. We always use version 4.1 to support macOS.
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

  // Now, let's go ahead and create our OpenGL context.
  render_context = SDL_GL_CreateContext(window);
  if (!render_context)
  {
    auto msg = std::format("Unable to create OpenGL context: {}", SDL_GetError());
    throw std::runtime_error(msg);
  }

  // Make the context current (it's the only context within the app).
  bool result = SDL_GL_MakeCurrent(window, render_context);
  if (!result)
  {
    auto msg = std::format("Unable to set primary render context: {}", SDL_GetError());
    throw std::runtime_error(msg);
  }

  // After, we'll load the OpenGL functiois pointer via glad.
  int version = gladLoadGLLoader((GLADloadproc)SDL_GL_GetProcAddress);
  if (version == 0)
    throw std::runtime_error("Unable to load OpenGL function pointers in context!");

  // Query the version of OpenGL that our context supports
  int versionMajor, versionMinor;
  glGetIntegerv(GL_MAJOR_VERSION, &versionMajor);
  glGetIntegerv(GL_MINOR_VERSION, &versionMinor);

  // Print a message out to confirm out success.
  is_active = true;
  log::out("Created nelo render context with OpenGL {}.{}", versionMajor, versionMinor);

  // We can also go ahead and setup blending since it is needed by all renderers.
  glEnable(GL_BLEND);
  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
}

void context::kill()
{
  if (!initialized)
    return;

  // We'll first destroy our OpenGL context.
  SDL_GL_DestroyContext(render_context);

  // Shutdown our window and shutdown SDL.
  SDL_DestroyWindow(window);
  SDL_Quit();
}

void context::poll()
{
  // If we are not active or headless, then we won't update anything.
  if (!initialized || !is_active || is_headless)
    return;

  // Poll all of the events have occured (probably none if running headless).
  SDL_Event event;
  while (SDL_PollEvent(&event))
  {
    if (event.type == SDL_EVENT_QUIT)
      is_active = false;
  }
}

void context::swap()
{
  if (!initialized || is_headless)
    return;

  SDL_GL_SwapWindow(window);
}

} // namespace nelo

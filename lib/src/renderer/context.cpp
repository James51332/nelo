#include "renderer/context.h"

#include <glad/glad.h>
#include <iostream>
#include <stdexcept>
#include <string>

namespace nelo
{

context::context(float w, float h, bool headless)
  : is_headless(headless), is_active(true)
{
  // Ensure that we only create only context per app.
  if (created_singleton)
    throw std::runtime_error("Unable to create multiple contexts per nelo instance!");
  created_singleton = true;

  // Initialize SDL library with our proper settings.
  SDL_Init(SDL_INIT_VIDEO);

  // Create our SDL_window with our desired flags.
  SDL_WindowFlags flags = SDL_WINDOW_OPENGL;

  // If we are headless, don't show the window.
  if (is_headless)
    flags |= SDL_WINDOW_HIDDEN;

  // Finally, we can go ahead and create our window.
  width = w;
  height = h;
  window = SDL_CreateWindow("nelo", width, height, flags);

  // If we fail, we can go ahead an throw an error.
  if (!window)
  {
    std::string msg = SDL_GetError();
    throw std::runtime_error("Unable to create an window required for context: " + msg);
  }

  // Set our flags for the OpenGL context. We always use version 4.1 to support macOS.
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 4);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 1);
  SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);

  // Now, let's go ahead and create our OpenGL context.
  render_context = SDL_GL_CreateContext(window);
  if (!render_context)
  {
    std::string msg = SDL_GetError();
    throw std::runtime_error("Unable to create OpenGL context: " + msg);
  }

  // Make the context current (it's the only context within the app).
  bool result = SDL_GL_MakeCurrent(window, render_context);
  if (!result)
  {
    std::string msg = SDL_GetError();
    throw std::runtime_error("Unabled to make OpenGL context current: " + msg);
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
  std::cout << "Created nelo render context with OpenGL " << versionMajor << "." << versionMinor
            << std::endl;

  // We can also go ahead and setup blending since it is needed by all renderers.
  glEnable(GL_BLEND);
  glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
}

context::~context()
{
  // We'll first destroy our OpenGL context.
  SDL_GL_DestroyContext(render_context);

  // Shutdown our window and shutdown SDL.
  SDL_DestroyWindow(window);
  SDL_Quit();

  // Don't mark the context singleton as being destroyed. Once per lifecycle.
  // created_singleton = false;
}

void context::update()
{
  // If we are not active, then we won't update anything.
  if (!is_active)
    return;

  // Poll all of the events have occured (probably none if running headless).
  SDL_Event event;
  while (SDL_PollEvent(&event))
  {
    if (event.type == SDL_EVENT_QUIT)
    {
      is_active = false;
    }
  }
}

void context::present()
{
  SDL_GL_SwapWindow(window);
}

} // namespace nelo

#include "context.h"

#include <stdexcept>
#include <string>

namespace nelo
{

context::context(bool headless)
  : is_headless(headless), is_active(true)
{
  // Ensure that we only create only context per app.
  if (created_singleton)
    throw new std::runtime_error("Unable to create multiple contexts per nelo instance!");
  created_singleton = true;

  // Initialize SDL library with our proper settings.
  SDL_Init(SDL_INIT_VIDEO);

  // Create our SDL_window with our desired flags.
  SDL_WindowFlags flags = SDL_WINDOW_HIGH_PIXEL_DENSITY | SDL_WINDOW_RESIZABLE | SDL_WINDOW_OPENGL;

  // If we are headless, don't show the window.
  if (is_headless)
    flags |= SDL_WINDOW_HIDDEN;

  // Finally, we can go ahead and create our window.
  window = SDL_CreateWindow("nelo", 800, 600, flags);

  // If we fail, we can go ahead an throw an error.
  if (!window)
  {
    std::string msg = SDL_GetError();
    throw new std::runtime_error("Unable to create an window required for context: " + msg);
  }
}

context::~context()
{
  // Shutdown our window and shutdown SDL.
  SDL_DestroyWindow(window);
  SDL_Quit();

  // Marked the context singleton as being destroyed.
  created_singleton = false;
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

} // namespace nelo

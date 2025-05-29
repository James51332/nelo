#include <glad/glad.h>
#include <iostream>
#include <nelo/renderer/context.h>
#include <nelo/scene/timeline.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  std::unique_ptr<nelo::context> context = std::make_unique<nelo::context>(false);

  // Update the window until we are closed, rendering via raw OpenGL for now.
  while (context->active())
  {
    // Poll events.
    context->update();

    // Render and present to the window.
    glClearColor(0.2f, 0.2f, 0.2f, 1.0f);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    context->present();
  }
}

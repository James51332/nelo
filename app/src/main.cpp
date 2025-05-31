#include <glad/glad.h>
#include <memory>
#include <nelo/renderer/context.h>
#include <nelo/renderer/renderer.h>
#include <nelo/scene/timeline.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  std::unique_ptr<nelo::context> context = std::make_unique<nelo::context>(false);

  // We also go ahead an create a renderer, which for now, can also be a unique ptr.
  std::unique_ptr<nelo::renderer> renderer = std::make_unique<nelo::renderer>();

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t;
  nelo::circle c = {.fill_color = glm::vec4(1.0f, 1.0f, 0.85f, 1.0f), .radius = 0.5f};

  // Update the window until we are closed, rendering via raw OpenGL for now.
  while (context->active())
  {
    // Poll events.
    context->update();

    // Render and present to the window.
    renderer->begin(glm::vec4(0.15f, 0.2f, 0.25f, 1.0f), 0.0f);
    renderer->submit(t, c);
    renderer->end();

    context->present();
  }
}

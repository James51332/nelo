#include <nelo/renderer/context.h>
#include <nelo/renderer/renderer.h>
#include <nelo/scene/timeline.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  nelo::context context(800, 600, false);

  // We also go ahead an create a renderer, which for now, can also be a unique ptr.
  nelo::renderer renderer(5);

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t = {.position = [](double t) -> glm::vec3
                       { return {3.0 * cos(3.0 * t), 3.0 * sin(3.0 * t), 0.0f}; }};
  nelo::circle c = {.fill_color = glm::vec4(1.0f, 1.0f, 0.85f, 1.0f), .radius = 1.0};

  // Update the window until we are closed, rendering via raw OpenGL for now.
  while (context.active())
  {
    // Poll events.
    context.update();

    // Render and present to the window.
    double time = SDL_GetTicks() / 1000.0;
    renderer.begin(glm::vec4(0.15f, 0.2f, 0.25f, 1.0f), time);
    renderer.submit(t, c);
    renderer.end();

    context.present();
  }
}

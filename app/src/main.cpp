#include <nelo/renderer/context.h>
#include <nelo/renderer/renderer.h>
#include <nelo/scene/timeline.h>
#include <nelo/video/encoder.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  nelo::context context(800, 600, false);

  // We also go ahead an create a renderer, which for now, can also be a unique ptr.
  nelo::renderer renderer(5);

  // We create an encoder to output this to video.
  nelo::encoder encoder(800, 600, 60, "output.mov");

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t = {.position = [](double t) -> glm::vec3
                       { return {3.0 * cos(3.0 * t), 3.0 * sin(3.0 * t), 0.0f}; }};
  nelo::circle c = {.fill_color = glm::vec4(1.0f, 1.0f, 0.85f, 1.0f), .radius = 1.0};

  // Update the window until we are closed, rendering via raw OpenGL for now.
  double time = 0.0f;
  while (context.active() && time <= 10.0f)
  {
    // Poll events.
    context.update();

    // Render and present to the window.
    time += 1 / 60.0f;
    renderer.begin(glm::vec4(0.15f, 0.2f, 0.25f, 1.0f), time);
    renderer.submit(t, c);
    renderer.end();

    context.present();

    // Submit to the encoder.
    encoder.submit();
  }

  encoder.end();
}

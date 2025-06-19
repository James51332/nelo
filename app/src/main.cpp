#include <glad/glad.h>
#include <nelo/renderer/circle_renderer.h>
#include <nelo/renderer/context.h>
#include <nelo/renderer/curve_renderer.h>
#include <nelo/scene/path.h>
#include <nelo/scene/timeline.h>
#include <nelo/video/encoder.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  nelo::context context(800, 600, false);

  // We also go ahead an create a renderer, which for now, can also be a unique ptr.
  nelo::circle_renderer circle_renderer(5);
  nelo::curve_renderer curve_renderer(5);

  // We create an encoder to output this to video.
  nelo::encoder encoder(800, 600, 60, "output.mov");

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t = {.position = [](double t) -> glm::vec3
                       { return {3.0 * cos(3.0 * t), 3.0 * sin(3.0 * t), 0.0}; }};
  nelo::circle c = {.fill_color = glm::vec4(1.0f, 1.0f, 0.85f, 1.0f), .radius = 1.0};

  nelo::curve curve = {.path = [](double t) -> glm::vec3 { return nelo::paths::square(t / 5.0); },
                       .color = glm::vec4(1.0f),
                       .end = 5.0};

  // Update the window until we are closed, rendering via raw OpenGL for now.
  double time = 0.0;
  while (context.active() && time <= 10.0)
  {
    // Poll events.
    context.update();

    // Clear the screen.
    glm::vec4 clear_color = glm::vec4(0.15f, 0.2f, 0.25f, 1.0f);
    glClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Update the time.
    time += 1 / 60.0;

    // Use our renderers. This will be handle by a scene system later.
    circle_renderer.begin(time);
    circle_renderer.submit(t, c);
    circle_renderer.end();

    // curve_renderer.begin(time);
    // curve_renderer.submit(t, curve);
    // curve_renderer.end();

    // Present the back buffer and submit to encoder.
    context.present();
    encoder.submit();
  }

  encoder.end();
}

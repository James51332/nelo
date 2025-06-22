#include <glad/glad.h>
#include <nelo/anim/timeline.h>
#include <nelo/core/context.h>
#include <nelo/core/encoder.h>
#include <nelo/core/log.h>
#include <nelo/render/circle_renderer.h>
#include <nelo/render/curve_renderer.h>
#include <nelo/script/server.h>
#include <nelo/types/path.h>

int main()
{
  // We can silence the console.
  nelo::log::use_console(true);

  // Let's test out lua. This should print a message from lua.
  nelo::server lua_server;

  // Create our nelo render context in a windowed mode.
  nelo::context context(800, 600, true);

  // We also go ahead and creatr our renderers. These will be unified in the future.
  nelo::circle_renderer circle_renderer(5);
  nelo::curve_renderer curve_renderer(5);

  // We create an encoder to output this to video.
  nelo::encoder encoder(800, 600, 60, "output.mov");

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t = {.position = [](double t) -> glm::vec3
                       { return {3.0 * cos(3.0 * t), 3.0 * sin(3.0 * t), 0.0}; }};
  nelo::circle c;
  nelo::curve curve = {.spline = [](double t) -> glm::vec3
                       { return {4.0 * cos(2.0 * t), 3.0 * sin(sin(3.0 * t)), 0.0}; },
                       .weight = [](double t) -> double { return 0.1 + 0.05 * sin(t * 5.0); },
                       .stroke = [](double t) -> nelo::color
                       { return glm::vec4(glm::vec3(0.7 * sin(4.0 * t)), 1.0); },
                       .end = 15.0,
                       .min_subdivisions = 8};

  // Update the window until we are closed.
  double time = 0.0;
  while (context.active() && time <= 10.0)
  {
    // Poll events.
    context.update();

    // Clear the screen.
    nelo::color clear_color = glm::vec4(0.15f, 0.2f, 0.25f, 1.0f);
    glClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Update the time.
    time += 1 / 60.0;

    // Use our renderers. This will be handle by a scene system later.
    circle_renderer.begin(time);
    circle_renderer.submit(t, c);
    circle_renderer.end();

    curve_renderer.begin(time);
    curve_renderer.submit(t, curve);
    curve_renderer.end();

    // Present the back buffer and submit to encoder.
    context.present();
    encoder.submit();
  }

  encoder.end();
}

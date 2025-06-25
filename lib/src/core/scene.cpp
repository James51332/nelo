#include "core/scene.h"

#include "core/context.h"
#include "core/encoder.h"
#include "render/circle_renderer.h"
#include "render/curve_renderer.h"

namespace nelo
{

scene::scene(const std::string& name)
  : scene_name(name)
{
}

entity scene::create_entity()
{
  return next_entity++;
}

void scene::destroy_entity(entity e)
{
  for (auto c : collections)
  {
    detail::collection_base col = std::any_cast<detail::collection_base>(c);
    if (col.exists(e))
      col.remove(e);
  }
}

void scene::play(double start, double end)
{
  // TODO These are some constants we want to eventually be configurable.
  constexpr static int width = 800;
  constexpr static int height = 600;
  constexpr static int fps = 60;
  constexpr static double scene_height = 5.0;
  constexpr static std::string file_ext = ".mov";
  constexpr static nelo::color clear_color = glm::vec4(0.15f, 0.2f, 0.25f, 1.0f);

  // Create our nelo render context in a headless mode. Eventually, this should be maintained
  // globally, and each scene should render to a framebuffer.
  nelo::context context(width, height, true);

  // Start by creating the encoder for the scene. We need to create a scene render to handle this at
  // somepoint.
  nelo::circle_renderer circle_renderer(scene_height);
  nelo::curve_renderer curve_renderer(scene_height);

  // We create an encoder to output this to video.
  std::filesystem::path output_path = scene_name + file_ext;
  nelo::encoder encoder(width, height, fps, output_path);

  // We can also create a timeline, and use it to drive our animation.
  nelo::transform t = {tmp};

  nelo::circle c;
  nelo::curve curve = {.spline = [](double t) -> glm::vec3
                       { return {4.0 * cos(2.0 * t), 3.0 * sin(sin(3.0 * t)), 0.0}; },
                       .weight = [](double t) -> double { return 0.1 + 0.05 * sin(t * 5.0); },
                       .stroke = [](double t) -> nelo::color
                       { return glm::vec4(glm::vec3(0.7 * sin(4.0 * t)), 1.0); },
                       .end = 15.0,
                       .min_subdivisions = 8};

  // Update the window until we are closed.
  double time = start;
  while (context.active() && time <= end)
  {
    // Poll events.
    context.update();

    // Clear the screen.
    glClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Update the time.
    time += 1 / static_cast<double>(fps);

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

} // namespace nelo

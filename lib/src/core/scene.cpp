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
  entity id = next_entity++;

  // Entities will come with a transform by default.
  add_component<transform>(id);

  return id;
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
    curve_renderer.begin(time);

    // TODO We are assuming that we still have transform. We need to enforce this by removing the
    // ability to remove components from entities, creating views that return entities with both
    // components, or both.
    auto& circle_col = get_collection<circle>();
    for (auto& pair : circle_col)
    {
      auto trans = get_component<transform>(pair.first);
      circle_renderer.submit(trans.sample(time), pair.second.sample(time));
    }

    auto& curve_col = get_collection<curve>();
    for (auto& pair : curve_col)
    {
      auto trans = get_component<transform>(pair.first);
      curve_renderer.submit(trans.sample(time), pair.second.sample(time));
    }

    circle_renderer.end();
    curve_renderer.end();

    // Present the back buffer and submit to encoder.
    context.present();
    encoder.submit();
  }

  encoder.end();
}

} // namespace nelo

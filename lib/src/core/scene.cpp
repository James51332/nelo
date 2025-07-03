#include "core/scene.h"

#include "core/encoder.h"
#include "render/circle_renderer.h"
#include "render/curve_renderer.h"
#include "render/framebuffer.h"

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
  // TODO These are some constants we want to eventually be configurable. We may want to move this
  // into some sort of scene renderer.
  constexpr static int fps = 60;
  constexpr static double scene_height = 5.0;
  constexpr static std::string file_ext = ".mov";
  constexpr static nelo::color clear_color = glm::vec4(0.15f, 0.2f, 0.25f, 1.0f);

  // Let's create a framebuffer with the scene width and height.
  framebuffer image(width, height);
  image.bind();
  glViewport(0, 0, width, height);

  circle_renderer circle_renderer(width, height, static_cast<float>(scene_height));
  curve_renderer curve_renderer(width, height, static_cast<float>(scene_height));

  // We create an encoder to output this to video.
  std::filesystem::path output_path = scene_name + file_ext;
  encoder encoder(static_cast<int>(width), static_cast<int>(height), fps, output_path);

  // Update the window until we are closed.
  double time = start;
  while (time <= end)
  {
    // Clear the screen.
    glClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    // Update the time.
    time += 1 / static_cast<double>(fps);

    // Use our renderers. This will be handle by a scene system later.
    circle_renderer.begin(time);
    curve_renderer.begin(time);

    // TODO We should create views to get all entities with both components.
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

    // Submit this frame to the encoder.
    encoder.submit();
  }

  // Complete the video write.
  encoder.end();
  image.unbind();
}

void scene::set_size(double width, double height)
{
  if (width < 0 || height < 0)
  {
    auto msg = std::format("Cannot set scene size to {} x {}!", width, height);
    throw std::runtime_error(msg);
  }

  this->width = static_cast<std::uint32_t>(width);
  this->height = static_cast<std::uint32_t>(height);
}

} // namespace nelo

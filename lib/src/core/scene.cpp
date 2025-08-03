#include "core/scene.h"

#include <format>

#include "render/circle_renderer.h"
#include "render/scene_renderer.h"
#include "types/transform.h"
#include "types/visibility.h"

namespace nelo
{

scene::scene(const std::string& name)
  : scene_name(name)
{
  // We'll add our default renderers here. User can remove these by fetching scene_renderer and
  // clearing.
  sc_renderer.add_renderer(std::make_shared<circle_renderer>());
  // sc_renderer.add_renderer(std::make_shared<curve_renderer>());
}

entity scene::create_entity()
{
  entity id = next_entity++;

  // Entities will always have a transform and visibility.
  add_component<transform>(id);
  add_component<visibility>(id);

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
  sc_renderer.play(*this, start, end);
}

void scene::set_size(double width, double height)
{
  if (width <= 0.0 || height <= 0.0)
  {
    auto msg = std::format("Unable to set scene size to {} x {}!", width, height);
    throw std::runtime_error(msg);
  }

  sc_renderer.set_size(static_cast<std::uint32_t>(width), static_cast<std::uint32_t>(height));
}

} // namespace nelo

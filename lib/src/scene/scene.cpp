#include "scene/scene.h"

namespace nelo
{

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

} // namespace nelo

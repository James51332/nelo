#pragma once

#include <any>
#include <typeindex>

#include "collection.h"
#include "entity.h"

namespace nelo
{

// Scene stores a collection of entities. Each entity is a key for a map to get timelines for
// various properties. Scene is also rensponsible for maintaining it's own global time.
class scene
{
public:
  scene();

  // Create and destroy entities
  entity create_entity();
  void destroy_entity(entity e);

  // Maintain the components for each identity
  template <typename T>
  timeline<T>& add_component(entity e, const T& component = T());

  template <typename T>
  timeline<T>& get_component(entity e);

  template <typename T>
  bool has_component(entity e);

  template <typename T>
  timeline<T>& remove_component(entity e);

  // This is an alternate way to modify the ECS. A collection is created the first time that this
  // method is called for a given type.
  template <typename T>
  collection<T>& get_collection();

  // Update global time
  void set_time(double time) { scene_time = time; }
  double get_time() const { return scene_time; }

private:
  // We start at one and move up.
  entity next_entity = 1;

  // The global time within the scene.
  double scene_time = 0.0;

  // We strictly enforce that each of these is of type collection<T>.
  std::unordered_map<std::type_index, std::any> collections;
};

template <typename T>
timeline<T>& scene::add_component(entity e, const T& component)
{
  return get_collection<T>().insert(e, component);
}

template <typename T>
timeline<T>& scene::get_component(entity e)
{
  return get_collection<T>().get(e);
}

template <typename T>
bool scene::has_component(entity e)
{
  return get_collection<T>().exists(e);
}

template <typename T>
timeline<T>& scene::remove_component(entity e)
{
  get_collection<T>().remove(e);
}

template <typename T>
collection<T>& scene::get_collection()
{
  auto index = std::type_index(typeid(T));

  if (collections.contains(index))
    collections[index] = collection<T>();

  return std::any_cast<collection<T>&>(collections[index]);
}

} // namespace nelo

#pragma once

#include <any>
#include <typeindex>

#include "core/collection.h"
#include "core/entity.h"

namespace nelo
{

// Scene stores a collection of entities. Each entity is a key for a map to get timelines for
// various properties. Scene is also rensponsible for maintaining it's own global time.
class scene
{
public:
  scene(const std::string& name);

  void set_name(const std::string& name) { scene_name = name; }
  const std::string& get_name() const { return scene_name; }

  // Create and destroy entities
  entity create_entity();
  void destroy_entity(entity e);

  // Maintain the components for each identity
  template <typename T>
  void add_component(entity e, const T& component = T());

  template <typename T>
  void add_component(entity e, timeline<T> component);

  template <typename T>
  timeline<T> get_component(entity e);

  template <typename T>
  bool has_component(entity e);

  template <typename T>
  timeline<T> remove_component(entity e);

  // This is an alternate way to modify the ECS. A collection is created the first time that this
  // method is called for a given type.
  template <typename T>
  collection<T>& get_collection();

  // Update global time
  void set_time(double time) { scene_time = time; }
  double get_time() const { return scene_time; }

  // Render the animation. TODO We need to implement framebuffers to handle any size, or rendering
  // with multiple scenes.
  void play(double start, double end);
  void play(double duration) { play(0.0, duration); }

private:
  // We start at one and move up. No harm making this static so entites can't accidentally be used
  // in wrong scene.
  inline static entity next_entity = 1;

  std::string scene_name;

  // The global time within the scene.
  double scene_time = 0.0;

  // We strictly enforce that each of these is of type collection<T>.
  std::unordered_map<std::type_index, std::any> collections;
};

template <typename T>
void scene::add_component(entity e, const T& component)
{
  get_collection<T>().emplace(e, component);
}

template <typename T>
void scene::add_component(entity e, timeline<T> component)
{
  get_collection<T>().insert(e, component);
}

template <typename T>
timeline<T> scene::get_component(entity e)
{
  return get_collection<T>().get(e);
}

template <typename T>
bool scene::has_component(entity e)
{
  return get_collection<T>().exists(e);
}

template <typename T>
timeline<T> scene::remove_component(entity e)
{
  get_collection<T>().remove(e);
}

template <typename T>
collection<T>& scene::get_collection()
{
  auto index = std::type_index(typeid(T));

  if (!collections.contains(index))
    collections[index] = collection<T>();

  return std::any_cast<collection<T>&>(collections.at(index));
}

} // namespace nelo

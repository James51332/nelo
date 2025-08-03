#pragma once

#include <any>
#include <typeindex>

#include "core/collection.h"
#include "core/entity.h"
#include "render/scene_renderer.h"
#include "types/transform.h"
#include "types/visibility.h"

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

  // Let's create a templated method that will find the intersection of get_component for a list of
  // types. It's better to return the entity id's than the components themselves. We also should
  // note that all entities are guaranteed to have a transform and visibility component, as this is
  // anticipated to be better for performance than to get a view with the correct components.
  template <typename T, typename... Args>
  std::vector<entity> get_view() const;

  // Render the animation. This can be called multiple times, so we can render multiple scenes, or
  // variations of scenes within the same script.
  void play(double start, double end);
  void play(double duration) { play(0.0, duration); }

  // Let us keep track of the scene render size. Use double for lua bindings
  void set_size(double width, double height);

  scene_renderer& get_renderer() { return sc_renderer; }

private:
  // We start at one and move up. No harm making this static so entites can't accidentally be used
  // in wrong scene.
  inline static entity next_entity = 1;

  std::string scene_name;

  // We strictly enforce that each of these is of type collection<T>.
  std::unordered_map<std::type_index, std::any> collections;

  scene_renderer sc_renderer;
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
  // Don't let us remove the transform or visibility component from an entity.
  if ((typeid(T) == typeid(transform)) || (typeid(T) == typeid(visibility)))
    throw std::runtime_error("Cannot remove transform or visibility component from entity!");

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

template <typename T, typename... Args>
std::vector<entity> scene::get_view() const
{
  // Let's start with the collection for type T. Then we'll add entities which have all of the other
  // types to our compiled list.
  collection<T> candidates = get_collection<T>();
  std::vector<entity> results;

  for (auto& pair : candidates)
  {
    // We'll use a fold over the parameter pack to skip this entity if any of the components do not
    // exist. TODO We should note that this is costly, and we can optimize by fetching collections
    // first. Using has_component has to fetch the container, which can be wasteful.
    entity id = pair.first;
    bool add = true;

    auto try_type = [id, &add](auto type_tag)
    {
      if (!has_compenent<decltype(type_tag)>(id))
        add = false;
    };
    (try_type(std::type_identity<Args>{}), ...);

    if (add)
      results.push_back(id);
  }

  // We can return the compiled vector.
  return std::move(results);
}

} // namespace nelo

#pragma once

#include <unordered_map>

#include "anim/timeline.h"
#include "core/entity.h"

namespace nelo
{

namespace detail
{

// This class is used to store all of the component caches in a
class collection_base
{
public:
  virtual bool exists(entity key)
  {
    throw std::runtime_error("Cannot check if entity exists in collection_base!");
  }

  virtual void remove(entity key)
  {
    throw std::runtime_error("Cannot remove from collection_base!");
  }
};

} // namespace detail

// A collection is a map of entity IDs to timelines.
template <typename T>
class collection : public detail::collection_base
{
public:
  collection() = default;

  void insert(entity key, timeline<T> val)
  {
    if (exists(key))
      map.erase(key);
    map.insert({key, val});
  }

  template <typename... Args>
  void emplace(entity key, Args&&... args)
  {
    if (exists(key))
      map.erase(key);
    map.emplace(key, std::forward<Args>(args)...);
  }

  timeline<T> get(entity key)
  {
    if (!exists(key))
      throw std::runtime_error("Cannot get element from collection because key is not in use");

    return map.at(key);
  }

  // We cannot return the timeline<T> because collection base needs remove.
  void remove(entity key)
  {
    if (!exists(key))
      throw std::runtime_error("Cannot remove element into collection because key is not in use");
    map.erase(key);
  }

  bool exists(entity key) { return map.contains(key); }

  // TODO We want to support views as well in the future. This is a temporary API.
  std::unordered_map<entity, timeline<T>>::iterator begin() { return map.begin(); }
  std::unordered_map<entity, timeline<T>>::iterator end() { return map.end(); }

private:
  std::unordered_map<entity, timeline<T>> map;
};

} // namespace nelo

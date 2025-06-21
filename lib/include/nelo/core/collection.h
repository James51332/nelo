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
  virtual bool exists(entity key);
  virtual void remove(entity key);
};

} // namespace detail

// A collection is a map of entity IDs to timelines.
template <typename T>
class collection : public detail::collection_base
{
public:
  collection() = default;

  void insert(entity key, const timeline<T>& map)
  {
    if (exists(key))
      throw std::runtime_error("Cannot insert element into collection because key is in use");
    map.insert(key, map);
  }

  void emplace(entity key, timeline<T>&& map)
  {
    if (exists(key))
      throw std::runtime_error("Cannot emplace element into collection because key is in use");
    map.emplace(key, std::move(map));
  }

  timeline<T> get(entity key)
  {
    if (!exists(key))
      throw std::runtime_error("Cannot get element from collection because key is not in use");

    return map.find(key);
  }

  timeline<T> remove(entity key)
  {
    if (!exists(key))
      throw std::runtime_error("Cannot remove element into collection because key is not in use");
    return map.erase(key);
  }

  bool exists(entity key) { return map.contains(key); }

private:
  std::unordered_map<entity, timeline<T>> map;
};

} // namespace nelo

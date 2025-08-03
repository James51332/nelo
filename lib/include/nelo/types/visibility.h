#pragma once

#include "anim/timeline.h"
#include "anim/traits.h"
#include "script/lua_traits.h"

namespace nelo
{

// A struct that maintains the visibility of an entity for all components.
struct visibility
{
  timeline<bool> hidden = false;
  timeline<double> opacity = 1.0;
  timeline<double> z_index = 0.0;
};

template <>
struct lua_traits<visibility>
{
  constexpr static auto name() { return "visibility"; };

  constexpr static auto fields()
  {
    return std::make_tuple(field("hidden", &visibility::hidden),
                           field("opacity", &visibility::opacity),
                           field("z_index", &visibility::z_index));
  }
};

template <>
struct timeline_traits<visibility>
{
  static visibility lerp(visibility a, visibility b, double t) = delete;
  static visibility add(visibility a, visibility b) = delete;
  static visibility multiply(visibility a, visibility b) = delete;
};

} // namespace nelo

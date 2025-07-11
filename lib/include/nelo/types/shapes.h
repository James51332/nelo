#pragma once

#include "anim/color.h"
#include "anim/timeline.h"
#include "script/lua_traits.h"

namespace nelo
{

struct circle
{
  timeline<double> radius = 1.0;
  timeline<color> fill_color = glm::vec4(1.0f);
};

template <>
struct lua_traits<circle>
{
  constexpr static auto name() { return "circle"; }

  constexpr static auto fields()
  {
    return std::make_tuple(field("radius", &circle::radius),
                           field("fill_color", &circle::fill_color));
  }
};

// TODO Fill in this methods
template <>
struct timeline_traits<circle>
{
  circle lerp(circle a, circle b, double t) = delete;
  circle add(circle a, circle b) = delete;
  circle multiple(circle a, circle b) = delete;
};

} // namespace nelo

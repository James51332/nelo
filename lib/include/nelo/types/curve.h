#pragma once

#include "anim/color.h"
#include "script/lua_traits.h"
#include "types/path.h"

namespace nelo
{

// A curve is a parametric curve which can be rendered in numerous ways.
struct curve
{
  // This very meta. Technically, curve is also stored as a timeline. We have a timeline of a
  // timeline of a timeline.
  timeline<path> spline;

  // Color of the path. This is linearly interpolated between subdivided points when rendering.
  timeline<path_property<color>> stroke = glm::vec4(1.0f);

  // Thickness of the line.
  timeline<path_property<double>> weight = 0.1;

  // These track the beginning and end of rendering.
  timeline<double> start = 0.0;
  timeline<double> end = -1.0;

  // If end is less than start, the path will be rendered from zero to the length of the path.
  // By default, a curve will ignore the transform of the entity, but it can be made to use it.
  timeline<bool> use_transform = false;

  // Subdivisions is the number of times that the line can be subdivided recursively, not total.
  // These values are casted to integers internally.
  timeline<double> min_subdivisions = 4;
  timeline<double> max_subdivisions = 10;

  // How far a point needs to be from the line connecting the points surrouding it in order for the
  // segments around it to be subdivided.
  timeline<double> threshold = 0.005;
};

// Declare the lua traits for this type.
template <>
struct lua_traits<curve>
{
  constexpr static auto name() { return "curve"; }

  constexpr static auto fields()
  {
    return std::make_tuple(field("spline", &curve::spline), field("stroke", &curve::stroke),
                           field("weight", &curve::weight), field("start", &curve::start),
                           field("end", &curve::end), field("use_transform", &curve::use_transform),
                           field("min_subdivisions", &curve::min_subdivisions),
                           field("max_subdivisions", &curve::max_subdivisions),
                           field("threshold", &curve::threshold));
  }
};

// TODO Fill in this methods
template <>
struct timeline_traits<curve>
{
  curve lerp(curve a, curve b, double t) = delete;
  curve add(curve a, curve b) = delete;
  curve multiple(curve a, curve b) = delete;
};

// We also want to be able to trace around a shape at any point to generate a curve. TODO Write an
// API to trace around a timeline<shape> or shape (should just cast to constant timeline).

} // namespace nelo

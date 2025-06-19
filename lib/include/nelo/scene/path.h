#pragma once

#include <functional>
#include <glm/glm.hpp>

#include "scene/timeline.h"
#include "scene/traits.h"

namespace nelo
{

// Some properties vary along a path such as color or stroke weight. We create this to reduce
// confusion as timelines become increasingly layered. As much of that complexity as possible should
// be hidden from the user unless theyopt to use it.
template <typename T>
using path_property = timeline<T>;

// Paths are timelines of position over time. They can be used to render curves, move objects, etc.
using path = path_property<glm::vec3>;

// These are parametric functions which should be evaluated between t = 0 and t = 1. They can be
// useful to define a path, but are not themselves paths. They are used internally to trace around
// shapes and generate path objects.
namespace paths
{

// Linearly interpolates between two positions. This one cannot be used as a path directly.
inline static glm::vec3 lerp(const glm::vec3& begin, const glm::vec3& end, double t)
{
  return timeline_traits<glm::vec3>::lerp(begin, end, t);
}

// Traces unit circle in counter clockwise direction. This technically can be used as a path
// directly.
inline static std::function<glm::vec3(double)> circle = [](double t)
{
  t *= 2.0 * M_PI;
  return glm::vec3(cos(t), sin(t), 0.0f);
};

// Generates the point at time t around a unit square beginning on the x-axis.
inline static std::function<glm::vec3(double)> square = [](double t)
{
  // We cache these points.
  constexpr static glm::vec3 right = glm::vec3(1.0, 0.0, 0.0);
  constexpr static glm::vec3 top_right = glm::vec3(1.0, 1.0, 0.0);
  constexpr static glm::vec3 top_left = glm::vec3(-1.0, 1.0, 0.0);
  constexpr static glm::vec3 bottom_left = glm::vec3(-1.0, 1.0, 0.0);
  constexpr static glm::vec3 bottom_right = glm::vec3(1.0, -1.0, 0.0);

  // The square has four sides, but we start at 3 o'clock and break it into eight pieces.
  t *= 4.0;

  // Go around each side of the square, and return the side we are on.
  if (t <= 0.5)
    return lerp(right, top_right, 2.0 * t);
  else if (t <= 1.5)
    return lerp(top_right, top_left, t - 0.5);
  else if (t <= 2.5)
    return lerp(top_left, bottom_left, t - 1.5);
  else if (t <= 3.5)
    return lerp(bottom_left, bottom_right, t - 2.5);
  else
    return lerp(bottom_right, right, 2.0 * t - 7.0);
};

} // namespace paths

// We'll also define the timeline_traits for path. These will be run every frame, so this could
// potentially get costly, since each frame we need to create a new timeline to evaluate at each
// point. TODO It might be worth it to create some real form of wrapper that can be downcasted, but
// I'm not sure how that would immediately help, since this API still requires a new path to be
// generated.
template <typename T>
struct timeline_traits<path_property<T>>
{
  static path_property<T> lerp(const path_property<T>& a, const path_property<T>& b, double t)
  {
    path_property<T> res = [&a, &b, t](double alpha) -> T
    { return timeline_traits<T>::lerp(a.sample(alpha), b.sample(alpha), t); };
    res.set_length(glm::min(a.length(), b.length()));
    return res;
  }

  static path_property<T> add(const path_property<T>& a, const path_property<T>& b)
  {
    path_property<T> res = [&](double alpha) -> T
    { return timeline_traits<T>::add(a.sample(alpha), b.sample(alpha)); };
    res.set_length(glm::min(a.length(), b.length()));
    return res;
  }

  static path_property<T> multiply(const path_property<T>& a, const path_property<T>& b)
  {
    path_property<T> res = [&](double alpha) -> T
    { return timeline_traits<T>::multiply(a.sample(alpha), b.sample(alpha)); };
    res.set_length(glm::min(a.length(), b.length()));
    return res;
  }
};

} // namespace nelo

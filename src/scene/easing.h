#pragma once

#include <functional>
#include <glm/glm.hpp>

namespace nelo
{

// We can use a simple using statement so it's easy to define custom easing functions when the
// provided aren't enough.
using easing_func = std::function<double(double)>;

namespace easing
{

// This is a non-standard function, but it might be a nice way to discretize keyframes. I might
// change this API later.
inline static easing_func step = [](double t) -> double { return 0; };

// Linear
inline static easing_func linear = [](double t) -> double { return t; };

// Quadratic
inline static easing_func inQuad = [](double t) -> double { return t * t; };
inline static easing_func outQuad = [](double t) -> double { return 1.0 - (1.0 - t) * (1.0 - t); };
inline static easing_func inOutQuad = [](double t) -> double
{ return t < 0.5 ? 2.0 * t * t : 1.0 - glm::pow(-2.0 * t + 2.0, 2.0) / 2.0; };

// Cubic
inline static easing_func inCubic = [](double t) -> double { return t * t * t; };
inline static easing_func outCubic = [](double t) -> double
{ return 1.0 - glm::pow(1.0 - t, 3.0); };
inline static easing_func inOutCubic = [](double t) -> double
{ return t < 0.5 ? 4.0 * t * t * t : 1.0 - glm::pow(-2.0 * t + 2.0, 3.0) / 2.0; };

} // namespace easing

} // namespace nelo

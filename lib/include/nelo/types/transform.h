#pragma once

#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

#include "anim/timeline.h"
#include "anim/traits.h"

namespace nelo
{

struct transform
{
  timeline<glm::vec3> position = glm::vec3(0.0f);
  timeline<glm::quat> rotation = glm::quat(1.0f, 0.0f, 0.0f, 0.0f);
  timeline<double> scale = 1.0;
};

// We also need to define timeline traits for this
template <>
struct timeline_traits<transform>
{
  // TODO implement these methods TODO switch to references wherever possible
  transform lerp(transform a, transform b, double t) = delete;
  transform add(transform a, transform b) = delete;
  transform multiply(transform a, transform b) = delete;
};

} // namespace nelo

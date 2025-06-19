#pragma once

#include "renderer/color.h"
#include "scene/timeline.h"

namespace nelo
{

struct circle
{
  timeline<double> radius = 1.0;
  timeline<color> fill_color = glm::vec4(1.0f);
};

} // namespace nelo

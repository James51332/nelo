#pragma once

#include "renderer/color.h"
#include "scene/timeline.h"

namespace nelo
{

struct circle
{
  timeline<color> fill_color;
  timeline<double> radius;
};

} // namespace nelo

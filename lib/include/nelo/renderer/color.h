#pragma once

#include <glm/glm.hpp>

namespace nelo
{

// For now, colors are just an alias for a glm::vec4. This means that the default modes of adding,
// multiplying or lerping will be the most trivial definitions. However, we may want to have
// blending modes for colors in the future, so I'm extracting this into a separate header to hopeful
// allow refactors in the future to be easier.
using color = glm::vec4;

} // namespace nelo

#pragma once

#include <sol/sol.hpp>

namespace nelo
{

class lua_scene
{
public:
  static void create_types(sol::state& lua);
};

} // namespace nelo

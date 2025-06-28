#pragma once

#include <sol/sol.hpp>

namespace nelo
{

class lua_scene
{
public:
  static void create_types(sol::state_view lua, sol::table binding);
};

} // namespace nelo

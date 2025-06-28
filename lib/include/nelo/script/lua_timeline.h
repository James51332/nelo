#pragma once

#include <sol/sol.hpp>

namespace nelo
{

// Helper struct that we use to allow timeline<T> to be used relatively seemlessly in lua. To get
// this to work, we have to do a lot of boilerplate, since types are compile-time in C++. This is
// accomplished internally using macros.
struct lua_timeline
{
  // Declares the lua type so that it can be used in the code.
  static void create_types(sol::state_view lua, sol::table binding);
};

} // namespace nelo

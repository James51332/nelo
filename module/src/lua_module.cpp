#include <nelo/core/context.h>
#include <nelo/script/lua_types.h>
#include <sol/sol.hpp>

// This is the entry point for when the library is loaded as a lua module.
extern "C" int luaopen_nelo(lua_State* L)
{
  // We need to create a nelo context. This is global, and will closed when the process terminates.
  nelo::context::create();

  // We create a view over the lua state to add via sol.
  sol::state_view lua(L);

  // We need to load our module as a table, so we don't pollute global.
  sol::table nelo = lua.create_table();
  nelo::lua_types::create_types(lua, nelo);

  // We also create a method to load globals if the user wants.
  nelo["setup_globals"] = [lua]()
  {
    // Don't set them up more than once.
    static bool has_setup_globals = false;
    if (has_setup_globals)
      return;

    // Nice and easy to do this again.
    nelo::lua_types::create_types(lua, lua.globals());
    has_setup_globals = true;
  };

  // We return the number of tables added to the lua stack.
  return sol::stack::push(lua, nelo);
}

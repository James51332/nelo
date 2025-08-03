#include "script/lua_server.h"

#include "script/lua_types.h"

namespace nelo
{

lua_server::lua_server()
{
  // Start by opening the base libraries in lua.
  lua.open_libraries(sol::lib::base, sol::lib::string, sol::lib::math, sol::lib::package);

  // Register our preload function.
  lua["package"]["preload"]["nelo"] = [](lua_State* L)
  {
    sol::state_view lua(L);

    sol::table mod = lua.create_table();
    lua_types::create_types(lua, mod);

    // Loading this table is slightly different. we'll automatically bind to the global table.
    sol::function bind_globals = mod["setup_globals"];
    bind_globals();

    return sol::stack::push(lua, mod);
  };
}

void lua_server::execute(const std::filesystem::path& path)
{
  lua.script_file(path.string());
}

} // namespace nelo

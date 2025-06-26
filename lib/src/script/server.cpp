#include "script/server.h"

#include "script/lua_types.h"

namespace nelo
{

server::server()
{
  lua.open_libraries(sol::lib::base, sol::lib::string, sol::lib::math);
  lua_types::create_types(lua);
}

void server::execute(const std::filesystem::path& path)
{
  lua.script_file(path.string());
}

} // namespace nelo

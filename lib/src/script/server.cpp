#include "script/server.h"

namespace nelo
{

server::server()
{
  lua.open_libraries(sol::lib::base, sol::lib::string, sol::lib::math);
  lua.script("print('Hello, lua')");
}

} // namespace nelo

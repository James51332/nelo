#include <glad/glad.h>
#include <nelo/script/lua_server.h>

int main()
{
  // Let's test out lua. This should render a scene using lua.
  nelo::lua_server lua_server;
  lua_server.execute("examples/hello_nelo.lua");
}

#include <nelo/core/context.h>
#include <nelo/core/log.h>
#include <nelo/script/lua_server.h>

#include <nelo/core/scene.h>
#include <nelo/types/shapes.h>

int main()
{
  // Create our nelo render context in a headless mode.
  nelo::context::create(1280, 720, false);

  // Since we are headless, we don't need to worry about polling OS events.
  nelo::lua_server lua_server;
  try
  {
    lua_server.execute("examples/hello_nelo.lua");
  }
  catch (std::exception e)
  {
    // TODO Redirect Sol3 logging to nelo::log.
    nelo::log::out("Error running lua script!");
  }

  // Clean up the context.
  nelo::context::kill();
}

#include "script/server.h"

#include <string>

#include "core/scene.h"

namespace nelo
{

server::server()
{
  lua.open_libraries(sol::lib::base, sol::lib::string, sol::lib::math);

  // We need to create our various types for lua. Let's start with the scene.
  scene_type = lua.new_usertype<scene>("scene", sol::constructors<scene(const std::string&)>());
  scene_type["play"] =
      sol::overload(static_cast<void (nelo::scene::*)(double)>(&nelo::scene::play),
                    static_cast<void (nelo::scene::*)(double, double)>(&nelo::scene::play));
}

void server::execute(const std::filesystem::path& path)
{
  lua.script_file(path.string());
}

} // namespace nelo

#include "script/server.h"

#include <string>

#include "core/scene.h"
#include "script/lua_types.h"

namespace nelo
{

server::server()
{
  lua.open_libraries(sol::lib::base, sol::lib::string, sol::lib::math);

  // We need to create our various types for lua. Let's start with the scene.
  auto scene_type =
      lua.new_usertype<scene>("scene", sol::constructors<scene(const std::string&)>());
  scene_type["play"] = sol::overload(static_cast<void (scene::*)(double)>(&scene::play),
                                     static_cast<void (scene::*)(double, double)>(&scene::play));
  scene_type["set_path"] = static_cast<void (scene::*)(timeline<glm::vec3>)>(&scene::set_path);

  lua_types::create_types(lua);
}

void server::execute(const std::filesystem::path& path)
{
  lua.script_file(path.string());
}

} // namespace nelo

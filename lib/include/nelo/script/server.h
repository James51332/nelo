#pragma once

#include <filesystem>
#include <sol/sol.hpp>

#include "core/scene.h"

namespace nelo
{

// A server is reponsible for executing nelo scripts. Each server has it's own state.
class server
{
public:
  server();

  void execute(const std::filesystem::path& path);

private:
  sol::state lua;
  sol::usertype<scene> scene_type;
};

} // namespace nelo

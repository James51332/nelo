#pragma once

#include <filesystem>
#include <sol/sol.hpp>

namespace nelo
{

// A server is reponsible for executing nelo scripts. Each server has it's own state.
class lua_server
{
public:
  lua_server();

  void execute(const std::filesystem::path& path);

private:
  sol::state lua;
};

} // namespace nelo

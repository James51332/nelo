#pragma once

#include <sol/sol.hpp>

namespace nelo
{

class server
{
public:
  server();

private:
  sol::state lua;
};

} // namespace nelo

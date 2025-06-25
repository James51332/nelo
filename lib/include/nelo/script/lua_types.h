#pragma once

#include <sol/sol.hpp>
#include <typeindex>

namespace nelo
{

// This class is the global source of truth regarding all lua types.
class lua_types
{
public:
  // Creates all lua types needed in nelo.
  static void create_types(sol::state& lua);

  // Deduce the type index for a type.
  static std::type_index deduce(sol::object obj);

  // Deduce the type from a string.
  static std::type_index type(const std::string& name);

  // Returns the type name for a lua type.
  static std::string name(std::type_index type);
};

} // namespace nelo

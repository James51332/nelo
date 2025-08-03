#pragma once

#include <glm/glm.hpp>
#include <sol/sol.hpp>
#include <typeindex>

#include "anim/color.h"
#include "types/curve.h"
#include "types/shapes.h"
#include "types/transform.h"
#include "types/visibility.h"

namespace nelo
{

// This class is the global source of truth regarding all lua types.
class lua_types
{
public:
  // Utility to all us to declare all of our lua core types in this header.
  template <typename... Ts>
  struct type_list
  {
  };

  // All core and component types to be declared in lua. These must have a lua_traits
  // specialization.
  using core_types = type_list<glm::vec2, glm::vec3, color, glm::quat>;
  using component_types = type_list<transform, circle, curve, visibility>;

public:
  // Creates all lua types needed in nelo.
  static void create_types(sol::state_view lua, sol::table binding);

  // Deduce the type index for a type.
  static std::type_index deduce(sol::object obj);

  // Deduce the type from a string.
  static std::type_index type(const std::string& name);

  // Returns the type name for a lua type.
  static std::string name(std::type_index type);

private:
  static void create_core_types(sol::state_view lua, sol::table binding);
};

} // namespace nelo

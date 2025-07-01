#pragma once

#include <glm/glm.hpp>
#include <tuple>
#include <typeinfo>
#include <utility>

#include "anim/color.h"

namespace nelo
{

// Lua traits will be similar to timeline traits. Any core type we want to expose to lua needs to
// implement lua_traits. This provides the name of the type, as well as the field names, so we can
// automatically create bindings in lua. Timelines are a major exception, so they are not handled
// through this API.
template <typename T>
struct lua_traits
{
  // Returns the name of the type as it is defined in lua.
  constexpr static auto name() { return typeid(T).name(); }

  // Returns a tuple of all of the fields as std::pairs (name, pointer to member). This is used to
  // initialize from a table or array in lua. If this is deleted, a type can only be default
  // constructed in lua, and no fields can be accessed.
  constexpr static auto fields();
};

// This is a helper method to generate an std::pair.
template <typename T, typename Member>
constexpr auto field(const char* name, Member T::* member)
{
  return std::pair{name, member};
}

// We can also provide specializations for the most basic type which we add to nelo. These are
// vectors, quaternions, colors. Eventually, we may implement this for other more sophisticated
// types.
template <>
struct lua_traits<glm::vec2>
{
  constexpr static auto name() { return "vec2"; }

  constexpr static auto fields()
  {
    return std::make_tuple(field("x", &glm::vec2::x), field("y", &glm::vec2::y));
  }
};

template <>
struct lua_traits<glm::vec3>
{
  constexpr static auto name() { return "vec3"; }

  constexpr static auto fields()
  {
    return std::make_tuple(field("x", &glm::vec3::x), field("y", &glm::vec3::y),
                           field("z", &glm::vec3::z));
  }
};

template <>
struct lua_traits<color>
{
  constexpr static auto name() { return "color"; }

  constexpr static auto fields()
  {
    return std::make_tuple(field("r", &color::r), field("g", &color::g), field("b", &color::b),
                           field("a", &color::a));
  }
};

template <>
struct lua_traits<glm::quat>
{
  constexpr static auto name() { return "quat"; }

  // We don't return any fields here. Rotations are not yet supported in lua.
  constexpr static auto fields() { return std::make_tuple(); }
};

} // namespace nelo

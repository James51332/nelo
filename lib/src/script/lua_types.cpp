#include "script/lua_types.h"

#include <glm/glm.hpp>
#include <stdexcept>
#include <unordered_map>

#include "script/lua_scene.h"
#include "script/lua_timeline.h"
#include "script/lua_traits.h"
#include "types/path.h"

#define CHECK_AND_RETURN_TYPE_INDEX(type, obj)                                                     \
  if (obj.is<type>())                                                                              \
    return typeid(type);

namespace nelo
{

// Utility to all us to declare all types at once.
void lua_types::create_types(sol::state_view lua, sol::table binding)
{
  // Start with the core types.
  lua_types::create_core_types(lua, binding);

  // Now, let's declare our timeline types.
  lua_timeline::create_types(lua, binding);

  // Create the ECS types.
  lua_scene::create_types(lua, binding);

  // We also create a method to load globals if the user wants.
  binding["setup_globals"] = [lua, binding]()
  {
    // Don't set them up more than once.
    static bool has_setup_globals = false;
    if (has_setup_globals)
      return;
    has_setup_globals = true;

    // Clone all properties on this table to the global table.
    sol::table globals = lua.globals();
    for (const auto& pair : binding)
    {
      sol::object key = pair.first;
      sol::object value = pair.second;

      // Only copy string keys to avoid indexing by number
      if (key.is<std::string>())
        globals.set(key, value);
    }

    has_setup_globals = true;
  };
}

std::type_index lua_types::deduce(sol::object obj)
{
  // From what I can find, there is not better way to deduce the type of an object than to just
  // check. Unfortunately we cannot use this at to construct lua_timelines, so we have to do this
  // check twice. We can best do this with macros.
  CHECK_AND_RETURN_TYPE_INDEX(bool, obj);
  CHECK_AND_RETURN_TYPE_INDEX(double, obj);
  CHECK_AND_RETURN_TYPE_INDEX(glm::vec2, obj);
  CHECK_AND_RETURN_TYPE_INDEX(glm::vec3, obj);
  CHECK_AND_RETURN_TYPE_INDEX(color, obj);
  CHECK_AND_RETURN_TYPE_INDEX(glm::quat, obj);
  CHECK_AND_RETURN_TYPE_INDEX(transform, obj);
  CHECK_AND_RETURN_TYPE_INDEX(circle, obj);
  CHECK_AND_RETURN_TYPE_INDEX(curve, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<bool>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<int>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<double>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<glm::vec2>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<glm::vec3>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<color>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<path>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<path_property<double>>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<path_property<color>>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<transform>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<circle>, obj);
  CHECK_AND_RETURN_TYPE_INDEX(timeline<curve>, obj);

  throw std::runtime_error("Unable to deduce type for lua object");
}

std::type_index lua_types::type(const std::string& name)
{
  const static std::unordered_map<std::string, std::type_index> type_map = {
      // All of the primitive types in the engine.
      {"bool",                          typeid(bool)                           },
      {"double",                        typeid(double)                         },
      {"number",                        typeid(double)                         }, // Number is alias for double.
      {"vec2",                          typeid(glm::vec2)                      },
      {"vec3",                          typeid(glm::vec3)                      },
      {"color",                         typeid(color)                          },
      {"quat",                          typeid(glm::quat)                      },

      // All of the component types in the engine.
      {"transform",                     typeid(transform)                      },
      {"circle",                        typeid(circle)                         },
      {"curve",                         typeid(curve)                          },

      // These are the timeline objects which make up all of the components.
      {"timeline_bool",                 typeid(timeline<bool>)                 },
      {"timeline_number",               typeid(timeline<double>)               },
      {"timeline_vec2",                 typeid(timeline<glm::vec2>)            },
      {"timeline_vec3",                 typeid(timeline<glm::vec3>)            }, // This is also a path.
      {"timeline_color",                typeid(timeline<color>)                },
      {"timeline_quat",                 typeid(glm::quat)                      },
      {"timeline_path",                 typeid(timeline<path>)                 },
      {"timeline_path_property_color",  typeid(timeline<path_property<color>>) },
      {"timeline_path_property_number", typeid(timeline<path_property<double>>)},

      // Timelines for all of the aggregate types.
      {"timeline_transform",            typeid(timeline<transform>)            },
      {"timeline_circle",               typeid(timeline<circle>)               },
      {"timeline_curve",                typeid(timeline<curve>)                }
  };

  if (type_map.contains(name))
    return type_map.at(name);

  std::string msg = std::format("Unable to deduce type \"{}\"", name);
  throw std::runtime_error(msg);
}

// TODO Migrate this to lua_traits<T> wherever possible.
std::string lua_types::name(const std::type_index type)
{
  // Type id has a name field. We still choose to map here for clarity, and mismatches.
  static std::unordered_map<std::type_index, std::string> type_map = {
      // All of the primitive types in the engine.
      {typeid(bool),                            "bool"                         },
      {typeid(double),                          "number"                       },
      {typeid(glm::vec2),                       "vec2"                         },
      {typeid(glm::vec3),                       "vec3"                         },
      {typeid(color),                           "color"                        },
      {typeid(glm::quat),                       "quat"                         },

      // All of the component types in the engine.
      {typeid(transform),                       "transform"                    },
      {typeid(circle),                          "circle"                       },
      {typeid(curve),                           "curve"                        },

      // These are the timeline objects which make up all of the components.
      {typeid(timeline<bool>),                  "timeline_bool"                },
      {typeid(timeline<double>),                "timeline_number"              },
      {typeid(timeline<glm::vec2>),             "timeline_vec2"                },
      {typeid(timeline<glm::vec3>),             "timeline_vec3"                }, // This is also a path.
      {typeid(timeline<color>),                 "timeline_color"               },
      {typeid(timeline<glm::quat>),             "timeline_quat"                },
      {typeid(timeline<path>),                  "timeline_path"                },
      {typeid(timeline<path_property<color>>),  "timeline_path_property_color" },
      {typeid(timeline<path_property<double>>), "timeline_path_property_number"},

      // Timelines for all of the aggregate types.
      {typeid(timeline<transform>),             "timeline_transform"           },
      {typeid(timeline<circle>),                "timeline_circle"              },
      {typeid(timeline<curve>),                 "timeline_curve"               }
  };

  // We just return the C++ name if we don't find it in our map.
  if (type_map.contains(type))
    return type_map.at(type);
  else
    return type.name();
}

// We need a helper method here. This allows us to bind a sol property if we are timeline. Otherwise
// we just return the member pointer. I don't think this will have much use elsewhere, so I'm just
// declaring it here.
template <typename T>
struct is_timeline : std::false_type
{
};

template <typename U>
struct is_timeline<timeline<U>> : std::true_type
{
};

// This helper method let's us assign a property to a table when the value can be either a
template <typename T, typename U>
static void assign_property(T& obj, const std::string& name, U T::* member, sol::object val)
{
  try
  {
    // If we're a timeline, see if we can assign ourself from a constant.
    if constexpr (is_timeline<U>::value)
    {
      using timeline_type = typename U::value_type;
      if (val.is<timeline_type>())
        obj.*member = timeline<timeline_type>(val.as<timeline_type>());
      else if (val.is<timeline<timeline_type>>())
        obj.*member = val.as<timeline<timeline_type>>();
      else
        throw;
    }
    else
      obj.*member = val.as<U>();
  }
  catch (...)
  {
    // Throw a pretty error so we can debug better.
    auto msg = std::format("Unable to assign to {} property", name);
    throw std::runtime_error(msg);
  }
}

// These are helper methods which use a lua_traits::fields methods to assign to an object from an
// array of set of key/value pairs.
template <typename T>
static void assign_from_array(T& obj, sol::table t)
{
  constexpr auto fields_list = lua_traits<T>::fields();
  std::size_t index = 1; // Lua starts with index one.

  // Apply our fields_list as a parameter pack to a lambda to use fold statement.
  std::apply(
      [&](auto&&... field)
      {
        // Apply our fold statement immediately.
        ((
             [&]
             {
               // We have no more values in our array. These should have values already.
               if (index > t.size())
                 return;

               auto name = field.first;
               auto member = field.second;
               sol::object val = t[index++];
               if (val.valid())
                 assign_property(obj, name, member, val);
             })(),
         ...);
      },
      fields_list);
}

template <typename T>
static void assign_from_table(T& obj, sol::table t)
{
  constexpr auto fields_list = lua_traits<T>::fields();

  std::apply(
      [&](auto&&... field)
      {
        ((
             [&]
             {
               auto name = field.first;
               auto member = field.second;
               sol::object val = t[name];
               if (val.valid())
                 assign_property(obj, name, member, val);
             })(),
         ...);
      },
      fields_list);
}

// Returns the member pointer, unless the type is a timeline. Then, it returns a sol::property which
// allows the timeline to be set from a constant or a timeline<T>.
template <typename T, typename FieldT>
static auto core_type_property(const std::string& name, FieldT T::* member)
{
  // If we are a simple type, just return the member pointer.
  if constexpr (!is_timeline<FieldT>::value)
    return member;
  else
    return sol::property([&name, member](T& self) { return self.*member; },
                         [&name, member](T& self, sol::object obj)
                         { assign_property(self, name, member, obj); });
}

template <typename T, typename... Args>
static void create_core_type(sol::state_view state, sol::table binding, Args&&... args)
{
  // To implement this method, we kinda use some template magic. Each type here has a fields<T>()
  // specialization that returns a tuple of pairs of name and member pointers to fields. We'll
  // flatten this tuple and pass to sol, as well as use it to enable initialization from a table
  // with or without named keys, automatically.
  constexpr auto name = lua_traits<T>::name();
  constexpr auto fields_list = lua_traits<T>::fields();

  // We'll support constructing from an array or a map. Check if the first element can be
  // accessed via index. Note that if a type doesn't have fields, this doesn't do anything. We
  // also support a regular ctor that applies to all fields. We need to check if a type has any
  // fields to do this.

  auto ctor = std::apply(
      [&](auto&&... fields)
      {
        auto no_arg_ctor = [](sol::this_state ts) { return T(); };
        auto all_arg_ctor = [](std::decay_t<decltype(std::declval<T>().*fields.second)>... args)
        { return T{args...}; };
        auto table_ctor = [](sol::table t)
        {
          T obj{};

          if (t[1].valid())
            assign_from_array(obj, t);
          else
            assign_from_table(obj, t);

          return obj;
        };

        // We'll have to default ctors if the type has no fields.
        if constexpr (std::tuple_size_v<decltype(fields_list)> == 0)
          return sol::factories(no_arg_ctor, table_ctor);
        else
          return sol::factories(no_arg_ctor, all_arg_ctor, table_ctor);
      },
      fields_list);

  // Now, we can create the type. We want to use T() instead of T.new(), so we'll use the default
  // constructor, then create a sol::call_constructor method that is overloaded with our nice new
  // methods. We need to again use std::apply to expand our flattened fields.
  auto type = binding.new_usertype<T>(name, sol::call_constructor, ctor, args...);

  // Now, we'll iterate over each of the fields and declare them as properties.
  std::apply(
      [&](auto&&... fields)
      {
        ((
             [&]
             {
               // We'll set each type using the name as the key, and a property from the member
               // pointer.
               type[fields.first] = core_type_property(fields.first, fields.second);
             })(),
         ...);
      },
      fields_list);
}

template <typename... Ts>
static void create_core_types_from_list(sol::state_view lua, sol::table binding,
                                        lua_types::type_list<Ts...>)
{
  (create_core_type<Ts>(lua, binding), ...);
}

void lua_types::create_core_types(sol::state_view lua, sol::table binding)
{
  create_core_types_from_list(lua, binding, core_types{});
  create_core_types_from_list(lua, binding, component_types{});
}

} // namespace nelo

#include "script/lua_types.h"

#include <glm/glm.hpp>
#include <stdexcept>
#include <unordered_map>

#include "anim/color.h"
#include "script/lua_timeline.h"
#include "types/curve.h"
#include "types/path.h"
#include "types/shapes.h"
#include "types/transform.h"

#define CHECK_AND_RETURN_TYPE_INDEX(type, obj)                                                     \
  if (obj.is<type>())                                                                              \
    return typeid(type);

namespace nelo
{

// This is a utility method that let's us bind the type of T to a timeline for any of timeline
// properties.
template <typename T, typename C>
auto timeline_property(timeline<T> C::* member)
{
  return sol::property([member](const C& self) -> const timeline<T>& { return self.*member; },
                       [member](C& self, const sol::object& o)
                       {
                         if (o.is<T>())
                         {
                           self.*member = timeline<T>(o.as<T>());
                         }
                         else if (o.is<timeline<T>>())
                         {
                           self.*member = o.as<timeline<T>>();
                         }
                         else
                         {
                           throw std::runtime_error("Invalid assignment to timeline property");
                         }
                       });
}

void lua_types::create_types(sol::state& lua)
{
  // TODO We wan some more ways to construct these objects. Maybe we want operators too.
  lua.new_usertype<glm::vec2>("vec2", sol::constructors<glm::vec2(float, float)>(), "x",
                              &glm::vec2::x, "y", &glm::vec2::y);

  lua.new_usertype<glm::vec3>("vec3", sol::constructors<glm::vec3(float, float, float)>(), "x",
                              &glm::vec3::x, "y", &glm::vec3::y, "z", &glm::vec3::z);

  lua.new_usertype<color>("color", sol::constructors<color(float, float, float, float)>(), "r",
                          &color::x, "g", &color::y, "b", &color::z, "a", &color::w);

  // TODO For now, we can just create quaternions from floats. We'll want an API around this.
  lua.new_usertype<glm::quat>("quat", sol::constructors<glm::quat(float, float, float, float)>());

  lua.new_usertype<transform>("transform", sol::constructors<transform()>(), "position",
                              timeline_property(&transform::position), "rotation",
                              timeline_property(&transform::rotation), "scale",
                              timeline_property(&transform::scale));

  lua.new_usertype<circle>("circle", sol::constructors<circle()>(), "radius",
                           timeline_property(&circle::radius), "fill_color",
                           timeline_property(&circle::fill_color));

  lua.new_usertype<curve>(
      "curve", sol::constructors<curve()>(), "spline", timeline_property(&curve::spline), "stroke",
      timeline_property(&curve::spline), "weight", timeline_property(&curve::weight), "start",
      timeline_property(&curve::start), "end", timeline_property(&curve::end), "use_transform",
      timeline_property(&curve::use_transform), "min_subdivisions",
      timeline_property(&curve::min_subdivisions), "max_subdivisions",
      timeline_property(&curve::max_subdivisions), "threshold",
      timeline_property(&curve::threshold));

  // Now, let's declare our timeline types.
  lua_timeline::create_types(lua);
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

std::string lua_types::name(std::type_index type)
{
  // Type id has a name field. We still choose to map here for clarity, and mismatches.
  const static std::unordered_map<std::type_index, std::string> type_map = {
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

} // namespace nelo

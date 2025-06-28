#include "script/lua_timeline.h"

#include <format>
#include <glm/glm.hpp>

#include "anim/color.h"
#include "anim/timeline.h"
#include "script/lua_types.h"
#include "types/curve.h"
#include "types/shapes.h"
#include "types/transform.h"

#define CHECK_AND_RETURN_LUA_TIMELINE(lua, check_type, index, lambda)                              \
  if (typeid(check_type) == index)                                                                  \
    return sol::make_object(                                                                       \
        lua, timeline<check_type>(                                                                 \
                 [lambda](double t) -> check_type                                                  \
                 {                                                                                 \
                   sol::object val = lambda(t);                                                    \
                   if (!val.is<check_type>())                                                      \
                   {                                                                               \
                     std::string msg =                                                             \
                         std::format("Function {} returned invalid value or did not return!",      \
                                     lua_types::name(typeid(timeline<check_type>)));               \
                     throw std::runtime_error(msg);                                                \
                   }                                                                               \
                                                                                                   \
                   return val.as<check_type>();                                                    \
                 }));

namespace nelo
{

// Declares the timeline_type for lua.
template <typename T>
auto declare_timeline_type(sol::table binding)
{
  std::string type_name = lua_types::name(typeid(timeline<T>));

  // This is the just the plain jane anchor ctor. However, we create a separate lambda so it's easy
  // to implement into our global ctor.
  auto anchor_ctor = [](const T& obj) -> timeline<T> { return timeline<T>(obj); };

  // We need a custom ctors that return shared_ptr's, and handle lua's type ambiguity.
  auto lambda_ctor = [](sol::function func) -> timeline<T>
  {
    // Copy capture to increase sol::function reference counter.
    return timeline<T>(
        [func](double t) -> T
        {
          sol::object val = func(t);
          if (!val.is<T>())
          {
            std::string msg = std::format("Function {} returned invalid value or did not return!",
                                          lua_types::name(typeid(timeline<T>)));
            throw std::runtime_error(msg);
          }

          return val.as<T>();
        });
  };

  // TODO Implement this.
  auto keyframe_ctor = [](sol::table table) { return timeline<T>(); };

  binding.new_usertype<timeline<T>>(type_name, sol::meta_function::construct,
                                    sol::overload(anchor_ctor, lambda_ctor, keyframe_ctor),
                                    "sample", &timeline<T>::sample);

  return anchor_ctor;
}

void lua_timeline::create_types(sol::state_view lua, sol::table binding)
{
  // Timelines can either be created from a timeline_T ctor, or by using the global timeline method.
  // The global timeline method will deduce the correct type for anchors and keyframes,
  // but requires a named type field for lambda timelines. Additionally, this name field can be
  // given to an anchor keyframe timeline. This gives us five official overloads.

  auto timeline_ctors = sol::overload(
      // We can have timelines of all of our primitive types.
      declare_timeline_type<bool>(binding), declare_timeline_type<double>(binding),
      declare_timeline_type<glm::vec2>(binding), declare_timeline_type<glm::vec3>(binding),
      declare_timeline_type<color>(binding),

      // We can also have timelines of all of our collection types.
      declare_timeline_type<transform>(binding), declare_timeline_type<circle>(binding),
      declare_timeline_type<curve>(binding),

      // We can have nested timelines for path properties.
      declare_timeline_type<timeline<path>>(binding),
      declare_timeline_type<timeline<path_property<double>>>(binding),
      declare_timeline_type<timeline<path_property<color>>>(binding),

      // We'll declare our lambda ctor as well.
      [lua](const std::string& name, sol::function lambda)
      {
        std::type_index type = lua_types::type(name);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, bool, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, double, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, glm::vec2, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, glm::vec3, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, color, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, transform, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, circle, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, curve, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, timeline<path>, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, timeline<path_property<double>>, type, lambda);
        CHECK_AND_RETURN_LUA_TIMELINE(lua, timeline<path_property<color>>, type, lambda);

        throw std::runtime_error("Unable to create timeline from function!");
      }

      // TODO We need to declare our keyframe ctor.
  );

  // Now, we can register a global timeline function.
  binding.set_function("timeline", timeline_ctors);
}

} // namespace nelo

#include "script/lua_scene.h"

#include "core/scene.h"
#include "script/lua_types.h"

namespace nelo
{

// The ECS in lua is masked as an object type. It's pretty nice. We can create an entity in a
// scene, and assign objects to it using various properties. This will feel like a struct type;
// it's super nifty. To do this, entities need to have a reference to the scene they were created
// in, so we create a light wrapper.
struct lua_entity
{
  entity id;
  sol::object owner; // Reference keeps scene alive.
};

// Helper method that adds components to entities as needed.
template <typename T>
static auto lua_entity_component()
{
  // We can rely on C++ to throw exceptions. These are just wrappers. This means we can only add a
  // component once. This is fine, at least for now.
  return sol::property([](lua_entity& self)
                       { return self.owner.as<scene&>().get_component<T>(self.id); },
                       [](lua_entity& self, const sol::object& obj)
                       {
                         auto& owner = self.owner.as<scene&>();
                         if (obj.is<T>())
                           owner.add_component<T>(self.id, obj.as<T>());
                         else if (obj.is<timeline<T>>())
                           owner.add_component<T>(self.id, obj.as<timeline<T>>());
                         else
                           throw std::runtime_error("Invalid assignment to timeline property");
                       });
}

template <typename T>
static void add_entity_component(sol::table binding)
{
  binding[lua_traits<T>::name()] = lua_entity_component<T>();
}

template <typename... Ts>
static void add_entity_components_from_list(sol::table binding, lua_types::type_list<Ts...>)
{
  (add_entity_component<Ts>(binding), ...);
}

void lua_scene::create_types(sol::state_view lua, sol::table binding)
{
  // Create our entity type. It will have properties which are defined by
  // lua_types::component_types.
  auto entity_type = binding.new_usertype<lua_entity>("entity", sol::no_constructor);
  add_entity_components_from_list(binding["entity"], lua_types::component_types{});

  // Declare the scene type as well. For now, I like if we don't enable deleting stuff. These
  // scripts are designed to by light an stateless. There's not a good reason I can think of to
  // create a removal API.
  auto scene_type = lua.new_usertype<scene>("scene", sol::call_constructor,
                                            sol::constructors<scene(const std::string&)>());

  // We can play for a duration over or over an interval.
  scene_type["play"] = sol::overload(static_cast<void (scene::*)(double)>(&scene::play),
                                     static_cast<void (scene::*)(double, double)>(&scene::play));

  // We can create an entity without any components. Alternatively we can give a table that will
  // fold over the fields of an entity and assign them.
  auto create_entity = [](sol::this_state ts, sol::object self) -> lua_entity
  {
    // Technically, we could call this statically with another scene, but the user should know
    // the consequence if they figure that out.
    if (!self.is<scene>())
      throw std::runtime_error("Unable to create entity! Please call this method using a colon.");

    sol::state_view state(ts);

    // self is already a Lua object; to safely hold a reference:
    sol::reference owner(state, self); // Makes a strong reference from the actual Lua object

    scene& cpp_scene = self.as<scene&>();
    return {cpp_scene.create_entity(), std::move(owner)};
  };

  scene_type["create_entity"] = sol::overload(
      create_entity,
      [&](sol::this_state ts, sol::object self, sol::table t)
      {
        // We'll still create an entity the same way. We need to assign from a table.
        // TODO While it doesn't make sense to assign from an array, I think it's not
        // a bad thing to support. For now, we'll just allow strict maps of component
        // names to their values.
        lua_entity e = create_entity(ts, self);

        // At this point, we can assume we have an actual scene, or we would've
        // thrown.
        auto& cpp_scene = self.as<scene&>();

        // For each of the properties of an entity, we need to check if there is a
        // key. We'll create a lambda function to deduce the parameter pack, and call
        // with the types. Then, we'll fold over that parameter pack using another
        // lambda that accepts one type, and checks if that key contains a value.
        [&]<typename... Ts>(lua_types::type_list<Ts...>)
        {
          (
              [&]<typename T>()
              {
                // Now, we'll check for the key.
                auto name = lua_traits<T>::name();
                sol::object obj = t[name];

                // If object is a timeline or a constant, we can add it. No need to throw a fit if a
                // key is not set.
                if (obj.is<T>())
                  cpp_scene.add_component<T>(e.id, obj.as<T>());
                else if (obj.is<timeline<T>>())
                  cpp_scene.add_component<timeline<T>>(e.id, obj.as<timeline<T>>());
              }.template operator()<Ts>(), // We need to call with the parameter pack as the type.
              ...);
        }(lua_types::component_types{});

        return std::move(e);
      });
}

} // namespace nelo

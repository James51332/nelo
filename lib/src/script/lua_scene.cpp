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
  scene& owner;
};

// Helper method that adds components to entities as needed.
template <typename T>
static auto lua_entity_component()
{
  // We can rely on C++ to throw exceptions. These are just wrappers. This means we can only add a
  // component once. This is fine, at least for now.
  return sol::property([](lua_entity& self) { return self.owner.get_component<T>(self.id); },
                       [](lua_entity& self, const sol::object& obj)
                       {
                         if (obj.is<T>())
                           self.owner.add_component<T>(self.id, obj.as<T>());
                         else if (obj.is<timeline<T>>())
                           self.owner.add_component<T>(self.id, obj.as<timeline<T>>());
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

  auto create_entity = [](scene& self) -> lua_entity { return {self.create_entity(), self}; };
  auto create_entity_overloaded =
      sol::overload(create_entity,
                    [&](scene& self, sol::table t)
                    {
                      // We can create an entity all the same and assign the traits.
                      lua_entity e = create_entity(self);

                      // For each of the properties of an entity, we need to check if there is a
                      // key. We'll create a lambda function to deduce the parameter pack, and fold
                      // over it to call inner lambda with each of the types.
                      [&]<typename... Ts>(lua_types::type_list<Ts...>)
                      {
                        (
                            [&]<typename T>()
                            {
                              // Now, we'll check for the key. T iterates over the types in the
                              // parameter pack.
                              auto name = lua_traits<T>::name();
                              sol::object obj = t[name];

                              // If object is a timeline or a constant, we can add it. First call
                              // constructs a constant timeline. Second uses specialization to
                              // insert timeline directly.
                              if (obj.is<T>())
                                self.add_component<T>(e.id, obj.as<T>());
                              else if (obj.is<timeline<T>>())
                                self.add_component<T>(e.id, obj.as<timeline<T>>());
                            }.template operator()<Ts>(), // We need to call with the parameter pack
                                                         // as the type.
                            ...);
                      }(lua_types::component_types{});

                      return std::move(e);
                    });

  // Declare the scene type as well. For now, I like if we don't enable deleting stuff. These
  // scripts are designed to by light an stateless. There's not a good reason I can think of to
  // create a removal API.
  auto scene_type = binding.new_usertype<scene>("scene", sol::call_constructor,
                                                sol::constructors<scene(const std::string&)>(),
                                                "create_entity", create_entity_overloaded);

  // We can play for a duration over or over an interval.
  scene_type["play"] = sol::overload(static_cast<void (scene::*)(double)>(&scene::play),
                                     static_cast<void (scene::*)(double, double)>(&scene::play));

  scene_type["set_size"] = &scene::set_size;
}

} // namespace nelo

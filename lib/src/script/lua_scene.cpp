#include "script/lua_scene.h"

#include "core/scene.h"
#include "types/curve.h"
#include "types/shapes.h"
#include "types/transform.h"

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
auto lua_entity_component()
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

void lua_scene::create_types(sol::state_view lua, sol::table binding)
{
  auto entity_type = binding.new_usertype<lua_entity>("entity", sol::no_constructor);
  entity_type["transform"] = lua_entity_component<transform>();
  entity_type["circle"] = lua_entity_component<circle>();
  entity_type["curve"] = lua_entity_component<curve>();

  // Declare the scene type as well. For now, I like if we don't enable deleting stuff. These
  // scripts are designed to by light an stateless. There's not a good reason I can think of to
  // create a removal API.
  auto scene_type = lua.new_usertype<scene>(
      "scene", sol::constructors<scene(const std::string&)>(), "play",
      sol::overload(static_cast<void (scene::*)(double)>(&scene::play),
                    static_cast<void (scene::*)(double, double)>(&scene::play)),
      "create_entity",
      [](sol::this_state ts, sol::object self) -> lua_entity
      {
        // Technically, we could call this statically with another scene, but the user should know
        // the consequence if they figure that out.
        if (!self.is<scene>())
          throw std::runtime_error(
              "Unable to create entity! Please call this method using a colon.");

        sol::state_view state(ts);

        // self is already a Lua object; to safely hold a reference:
        sol::reference owner(state, self); // Makes a strong reference from the actual Lua object

        scene& cpp_scene = self.as<scene&>();
        return {cpp_scene.create_entity(), std::move(owner)};
      });
}

} // namespace nelo

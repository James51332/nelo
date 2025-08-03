#pragma once

#include "core/scene.h"
#include "render/command.h"

namespace nelo
{

// This is the base class for all renderers in nelo. It's super easy to add new renderers. Whenever
// we create a scene, we add all of the classes that implement this API that we want. We can
// implement the defaults, or allow advanced users to create custom renderers. I'd like to
// eventually expose this to lua, but that is out of scope for quite some time.
class renderer
{
public:
  virtual void generate_commands(command_buffer& buffer, scene& state, double t) = 0;
};

} // namespace nelo

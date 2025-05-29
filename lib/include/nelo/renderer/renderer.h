#pragma once

#include <glm/glm.hpp>

namespace nelo
{

// The renderer is our public API for rendering. For now, we are only going to support rendering
// very basic stuff, so we only have the one class. Each render pipeline may have a seperate
// renderer (e.g shapes, text, paths, 3D). This can interop with the ECS once we have it. This
// renderer might turn into the shapes renderer.
class renderer
{
public:
  renderer();
  ~renderer();

  // This is our public facing renderer API.
  void circle(const glm::mat4& transform = glm::mat4(1.0f), double radius = 0.5f,
              const glm::vec4& color = glm::vec4(1.0f));

  // TODO We will replace this with a sprite API soon.
  void rect(const glm::vec2& position = glm::vec2(0.0f), float width = 1.0f, float height = 1.0f,
            const glm::vec4& color = glm::vec4(1.0f), const glm::mat4& transform = glm::mat4(1.0f));

private:
};

} // namespace nelo

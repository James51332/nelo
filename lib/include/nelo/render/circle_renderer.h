#pragma once

#include <glad/glad.h>
#include <glm/glm.hpp>

#include "render/buffer.h"
#include "render/renderer.h"

namespace nelo
{

// The renderer is our public API for rendering. For now, we are only going to support rendering
// very basic stuff, so we only have the one class. Each render pipeline may have a seperate
// renderer (e.g shapes, text, paths, 3D). This can interop with the ECS once we have it. This
// renderer might turn into the shapes renderer.
class circle_renderer : public renderer
{
public:
  // The renderer is the owner of the context for the current design.
  circle_renderer();
  ~circle_renderer();

  // This renderer is batched, so we declare when to begin and end a batch.
  void generate_commands(command_buffer& buffer, scene& state, double t);

private:
  // Simple struct that we can send to GPU to draw circles.
  struct sprite_vertex
  {
    glm::vec3 position;
    glm::vec4 color;
    glm::vec2 uv;
  };

  // To batch circles, we'll store a list of vertices and indices, then copy them into the buffer
  // before drawing.
  const int max_circles = 10000;
  const int max_vertices = max_circles * 4;
  const int max_indices = max_circles * 6;
  int num_circles = 0;
  sprite_vertex* vertex_array = nullptr;

  // For now, we can just store the raw GL types to get a prototype up and running ASAP.
  GLuint program;
  vertex_layout layout;
  buffer_pool pool;
  std::shared_ptr<buffer> ibo;
};

} // namespace nelo

#pragma once

#include <glad/glad.h>
#include <glm/glm.hpp>
#include <vector>

#include "render/buffer.h"
#include "render/renderer.h"

namespace nelo
{

// Renderer for circle components. Batches wherever possible. Added to scene_renderer by default.
class circle_renderer : public renderer
{
public:
  circle_renderer();
  ~circle_renderer();

  // This renderer is batched, so we declare when to begin and end a batch.
  void generate_commands(command_buffer& buffer, scene& state, double t);

public:
  // Simple struct that we can send to GPU to draw circles.
  struct sprite_vertex
  {
    glm::vec3 position;
    glm::vec4 color;
    glm::vec2 uv;
  };

private:
  // To batch circles, we'll store a list of vertices and indices, then copy them into the buffer
  // before drawing.
  const int max_circles = 10000;
  const int max_vertices = max_circles * 4;
  const int max_indices = max_circles * 6;
  int num_circles = 0;
  std::vector<sprite_vertex> vertex_array;

  // For now, we can just store the raw GL types to get a prototype up and running ASAP.
  GLuint program;
  vertex_layout layout;
  buffer_pool pool;
  std::shared_ptr<buffer> ibo;
};

} // namespace nelo

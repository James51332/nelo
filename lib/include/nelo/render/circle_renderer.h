#pragma once

#include <glad/glad.h>
#include <glm/glm.hpp>

#include "types/shapes.h"
#include "types/transform.h"

namespace nelo
{

// The renderer is our public API for rendering. For now, we are only going to support rendering
// very basic stuff, so we only have the one class. Each render pipeline may have a seperate
// renderer (e.g shapes, text, paths, 3D). This can interop with the ECS once we have it. This
// renderer might turn into the shapes renderer.
class circle_renderer
{
public:
  // The renderer is the owner of the context for the current design.
  circle_renderer(float scene_height = 5.0);
  ~circle_renderer();

  // This renderer is batched, so we declare when to begin and end a batch.
  void begin(double t);
  void end();

  // This is our public facing renderer API. We'll restructure as we design ECS.
  void submit(const transform& trans, const circle& obj);

private:
  // If we are full too early, then we need to flush instead of forcing the user to draw more stuff.
  void flush();

private:
  // Value to check whether the renderer is currently active. We cannot render unless we begin a
  // batch, even though we are internally drawing immediately for the time being.
  bool is_recording = false;
  double cur_time = 0.0;

  // The actual window height of the renderer.
  float scene_height = 0.0f;

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
  GLuint circle_program;
  GLuint circle_vao;
  GLuint circle_vbo;
  GLuint circle_ibo;
};

} // namespace nelo

#include "render/circle_renderer.h"

#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtx/quaternion.hpp>
#include <stdexcept>

#include "core/context.h"
#include "render/shaders.h"

namespace nelo
{

circle_renderer::circle_renderer(float scene_height)
  : scene_height(scene_height)
{
  // Load our shaders
  circle_program = shaders::load("circle_vertex.glsl", "circle_fragment.glsl");

  // Now, we gotta do some friggin maneuvering to create a vao.
  glGenVertexArrays(1, &circle_vao);

  // Create the vertex buffer and whatnot.
  vertex_array = new sprite_vertex[max_vertices];
  glGenBuffers(1, &circle_vbo);

  // Bind our vbo and add some data.
  glBindVertexArray(circle_vao);
  glBindBuffer(GL_ARRAY_BUFFER, circle_vbo);
  glBufferData(GL_ARRAY_BUFFER, sizeof(sprite_vertex) * max_vertices, nullptr, GL_DYNAMIC_DRAW);

  // Create the index buffer and whatnot. It can be statically generated.
  std::uint32_t* index_array = new std::uint32_t[max_indices];
  for (int i = 0; i < max_circles; ++i)
  {
    int first_vert = i * 4;
    int first_index = i * 6;

    index_array[first_index + 0] = first_index + 0;
    index_array[first_index + 1] = first_index + 1;
    index_array[first_index + 2] = first_index + 2;
    index_array[first_index + 3] = first_index + 0;
    index_array[first_index + 4] = first_index + 2;
    index_array[first_index + 5] = first_index + 3;
  }

  // Generate the ibo (it will be attached to the vao).
  glGenBuffers(1, &circle_ibo);
  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, circle_ibo);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(std::uint32_t) * max_indices, index_array,
               GL_STATIC_DRAW);

  // We can free this jit yo.
  delete[] index_array;

  // Let's setup our vertex attribs so that the shader doesn't throw a friggin fit.
  glEnableVertexAttribArray(0);
  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(sprite_vertex), (void*)0);
  glEnableVertexAttribArray(1);
  glVertexAttribPointer(1, 4, GL_FLOAT, GL_FALSE, sizeof(sprite_vertex), (void*)sizeof(glm::vec3));
  glEnableVertexAttribArray(2);
  glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, sizeof(sprite_vertex),
                        (void*)(sizeof(glm::vec3) + sizeof(glm::vec4)));
}

circle_renderer::~circle_renderer()
{
  // Who wants to friggin leak something. NOT me.
  delete[] vertex_array;
  glDeleteProgram(circle_program);
  glDeleteBuffers(1, &circle_vbo);
  glDeleteBuffers(1, &circle_ibo);
  glDeleteVertexArrays(1, &circle_vao);
}

void circle_renderer::begin(double t)
{
  if (is_recording)
    throw std::runtime_error("Attempted to begin render batch while another batch is active!");

  is_recording = true;
  cur_time = t;
}

void circle_renderer::end()
{
  if (!is_recording)
    throw std::runtime_error("Attempted to end render batch when no batch was active");

  is_recording = false;
  flush();
}

void circle_renderer::submit(const transform& transform, const circle& circle)
{
  // Make sure that we are actually in a thing.
  if (!is_recording)
    throw std::runtime_error("Cannot submit draw data while no batch is active!");

  // Store the vertices of a standard triangle that we are transforming. We add a little extra space
  // to give soft edges to the circle.
  constexpr static sprite_vertex vertices[] = {
      {{1.1f, 1.1f, 0.0f},   {1.0f, 1.0f, 1.0f, 1.0f}, {1.1f, 1.1f}  },
      {{1.1f, -1.1f, 0.0f},  {1.0f, 1.0f, 1.0f, 1.0f}, {1.1f, -1.1f} },
      {{-1.1f, -1.1f, 0.0f}, {1.0f, 1.0f, 1.0f, 1.0f}, {-1.1f, -1.1f}},
      {{-1.1f, 1.1f, 0.0f},  {1.0f, 1.0f, 1.0f, 1.0f}, {-1.1f, 1.1f} }
  };

  // Ensure that we don't run out of space to draw. If we do, secretly flush the batch.
  if (num_circles == max_circles)
    flush();

  // We cannot upload a transform to the GPU for every circle, so we apply the transform here and
  // now. We also adjust according to the radius of the circle. Let's fetch and build the values we
  // need.
  float radius = static_cast<float>(circle.radius.sample(cur_time));
  color col = circle.fill_color.sample(cur_time);
  glm::vec3 pos = transform.position.sample(cur_time);
  glm::quat rot = transform.rotation.sample(cur_time);
  float scale = static_cast<float>(transform.scale.sample(cur_time));

  // This is definetly costly, but it's not a huge deal for now.
  glm::mat4 mat = glm::translate(glm::mat4(1.0f), pos) * glm::toMat4(rot);

  int first_vertex = num_circles * 4;
  for (int i = 0; i < 4; i++)
  {
    // Store the index of the vertex we're modifying.
    int vert = first_vertex + i;

    // Make the changes we need.
    glm::vec4 pos = mat * glm::vec4(scale * radius * vertices[vert].position, 1.0);
    vertex_array[vert].position = glm::vec3(pos / pos.w);
    vertex_array[vert].color = col;
    vertex_array[vert].uv = vertices[vert].uv;
  }

  // Now, we just increment the number of circles that we have.
  num_circles++;
}

void circle_renderer::flush()
{
  // This method is private, so we don't need to verify that we are in a frame. However, we can
  // still return if there are no circles to draw.
  if (num_circles == 0)
    return;

  // A super easy way to get started with having a nice viewport is to just fetch and upload some
  // uniforms each frame.
  glUseProgram(circle_program);
  glUniform2f(glGetUniformLocation(circle_program, "viewport_size"), context::width,
              context::height);
  glUniform1f(glGetUniformLocation(circle_program, "scene_height"), scene_height);

  // Update our vertex buffer with the new data.
  glBindVertexArray(circle_vao);
  glBindBuffer(GL_ARRAY_BUFFER, circle_vbo);
  glBufferSubData(GL_ARRAY_BUFFER, 0, sizeof(sprite_vertex) * num_circles * 4, vertex_array);

  // Submit the draw call!
  glDrawElements(GL_TRIANGLES, num_circles * 6, GL_UNSIGNED_INT, 0);

  // Reset the number of circles.
  num_circles = 0;
}

} // namespace nelo

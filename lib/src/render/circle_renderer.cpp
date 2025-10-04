#include "render/circle_renderer.h"

#define GLM_ENABLE_EXPERIMENTAL
#include <glm/gtx/quaternion.hpp>

#include "render/shaders.h"
#include "types/shapes.h"
#include "types/transform.h"
#include "types/visibility.h"

namespace nelo
{

circle_renderer::circle_renderer()
{
  // Load our shaders and set our layout.
  program = shaders::load("circle_vertex.glsl", "circle_fragment.glsl");
  layout = {
      {"position", GL_FLOAT, 3},
      {"color",    GL_FLOAT, 4},
      {"uv",       GL_FLOAT, 2}
  };

  // Create the vertex buffer and whatnot.
  vertex_array = std::vector<sprite_vertex>(10000);

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
  ibo = std::make_shared<buffer>(GL_ELEMENT_ARRAY_BUFFER, max_indices * sizeof(std::uint32_t));

  // We can free this jit yo.
  delete[] index_array;
}

circle_renderer::~circle_renderer()
{
  // TODO We should create a system to automatically delete shaders.
  glDeleteProgram(program);
}

void circle_renderer::generate_commands(command_buffer& cmd_buffer, scene& state, double t)
{
  // Our goal is to batch whatever circles we can as possible. First, we'll group them into buckets
  // of the same z-index.
  auto collection = state.get_collection<circle>();
  std::unordered_map<double, std::vector<entity>> buckets;
  for (const auto& [entity, circle] : collection)
  {
    // Get the entity and skip it if the visibility is hidden.
    auto vis = state.get_component<visibility>(entity).sample(t);
    if (vis.hidden.sample(t))
      continue;

    // Place the circle into a bucket.
    double z_index = vis.z_index.sample(t);
    buckets[z_index].push_back(entity);
  }

  // Store the vertices of a standard triangle that we are transforming. We add a little extra space
  // to give soft edges to the circle.
  constexpr static sprite_vertex vertices[] = {
      {{1.1f, 1.1f, 0.0f},   {1.0f, 1.0f, 1.0f, 1.0f}, {1.1f, 1.1f}  },
      {{1.1f, -1.1f, 0.0f},  {1.0f, 1.0f, 1.0f, 1.0f}, {1.1f, -1.1f} },
      {{-1.1f, -1.1f, 0.0f}, {1.0f, 1.0f, 1.0f, 1.0f}, {-1.1f, -1.1f}},
      {{-1.1f, 1.1f, 0.0f},  {1.0f, 1.0f, 1.0f, 1.0f}, {-1.1f, 1.1f} }
  };

  // Let's reset our pool acquire a vbo. We'll acquire another if we fill up.
  pool.reset();
  std::shared_ptr<buffer> vbo = pool.acquire();

  // Now, we can iterate over the buckets and submit draw calls.
  for (auto& [z_index, bucket] : buckets)
  {
    // Get the bucket and its z_index.
    // Track how many entities we have batched for this bucket since last sent.
    std::size_t num_batched = 0;
    auto flush_batch = [&]()
    {
      // Don't worry about drawing if we have no circles in the batch.
      if (num_batched == 0)
        return;

      // Update our vbo with the vertex data.
      constexpr std::size_t vert_size = sizeof(sprite_vertex);
      std::size_t offset = vert_size * num_circles * 4;
      std::size_t num_bytes = vert_size * num_batched * 4;
      vbo->set_bytes(offset, num_bytes, vertex_array.data() + offset);

      // Submit the draw call.
      draw_command command;
      command.shader = program;
      command.layout = layout;
      command.vbo = vbo;
      command.ibo = ibo;
      command.index_offset = num_circles * 6;
      command.index_count = num_batched * 6;
      command.z_index = z_index;
      cmd_buffer.submit(command);

      // We should update the starting index and reset batch count.
      num_circles += num_batched;
      num_batched = 0;
    };

    // Encode all of the entities into a command buffer and encode the draw command.
    for (auto entity : bucket)
    {
      // Get the components from the entity.
      auto circ = state.get_component<circle>(entity).sample(t);
      auto trans = state.get_component<transform>(entity).sample(t);
      auto vis = state.get_component<visibility>(entity).sample(t);

      // Get our info for the circle.
      float radius = static_cast<float>(circ.radius.sample(t));
      color col = circ.fill_color.sample(t);
      glm::vec3 pos = trans.position.sample(t);
      glm::quat rot = trans.rotation.sample(t);
      float scale = static_cast<float>(trans.scale.sample(t));

      // Scale our color opacity by visibility opacity.
      col.a *= vis.opacity.sample(t);

      // Generate our transformation matrix.
      glm::mat4 mat = glm::translate(glm::mat4(1.0f), pos) * glm::toMat4(rot);

      int first_vertex = num_circles * 4;
      for (int i = 0; i < 4; i++)
      {
        // Store the index of the vertex we're modifying.
        int vert = first_vertex + i;

        // Make the changes we need.
        glm::vec4 pos = mat * glm::vec4(scale * radius * vertices[vert].position, 1.0);
        vertex_array[vert].position = vertices[vert].position; // glm::vec3(pos / pos.w);
        vertex_array[vert].color = col;
        vertex_array[vert].uv = vertices[vert].uv;
      }

      // Now, we just increment the number of batched that we have.
      num_batched++;

      // We should check and see if we are at the max number of circles. If we are, we should flush
      // the batch, and begin a new batch in a new VBO.
      if (num_circles + num_batched >= max_circles)
      {
        flush_batch();
        vbo = pool.acquire();
      }
    }

    // After we have completely finished the batch, we should flush.
    flush_batch();
  }
}

} // namespace nelo

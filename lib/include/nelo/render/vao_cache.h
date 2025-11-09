#pragma once

#include <glad/glad.h>
#include <stdexcept>
#include <unordered_map>

#include "render/buffer.h"
#include "render/layout.h"

namespace nelo
{

// To allow our renderers to think exclusively in terms of shaders and buffers, we create the
// following. It's a cache which ad hoc generates VAOs based on shader, buffers, and layout as a
// key. This will be removed once we support a different render API.
class vao_cache
{
  struct vao_key
  {
    GLuint vbo;
    GLuint ibo;
    GLuint shader;
    vertex_layout layout;

    bool operator==(const vao_key& other) const
    {
      return vbo == other.vbo && ibo == other.ibo && shader == other.shader &&
             layout == other.layout;
    }
  };

  // Define a private internal hash since this class is not public.
  struct vao_key_hash
  {
    std::size_t operator()(const vao_key& key) const
    {
      std::size_t h1 = std::hash<GLuint>()(key.vbo);
      std::size_t h2 = std::hash<GLuint>()(key.ibo);
      std::size_t h3 = std::hash<GLuint>()(key.shader);
      std::size_t h4 = std::hash<vertex_layout>()(key.layout);
      return ((h1 ^ (h2 << 1)) >> 1) ^ (h3 << 1) ^ (h4 << 2);
    }
  };

public:
  GLuint get_or_create(const std::shared_ptr<buffer>& vbo, const std::shared_ptr<buffer>& ibo,
                       GLuint shader, const vertex_layout& layout)
  {
    vao_key key{vbo->object(), ibo->object(), shader, layout};

    // If we have a VAO for this combo, we are rocking.
    if (cache.contains(key))
      return cache.at(key);

    // Otherwise, we should create it.
    GLuint vao;
    glGenVertexArrays(1, &vao);
    glBindVertexArray(vao);

    // Let's make sure that we have the correct types of buffers as we bind.
    if (vbo->buffer_type() != GL_ARRAY_BUFFER)
      throw std::runtime_error("Unable to bind vbo of incorrect type to vao!");
    glBindBuffer(GL_ARRAY_BUFFER, vbo->object());

    if (ibo->buffer_type() != GL_ELEMENT_ARRAY_BUFFER)
      throw std::runtime_error("Unable to bind ibo of incorrect type to vao!");
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ibo->object());

    // Now, we need to define the vertex layout. Do this in two passes.
    std::size_t stride = 0;
    for (auto element : layout.elements)
      stride += vertex_layout::element_size(element.type) * element.count;

    // Second pass binds now that we have stride info. Computation is slightly redundant.
    std::size_t slot = 0;
    std::size_t offset = 0;
    for (auto element : layout.elements)
    {
      glEnableVertexAttribArray(slot);
      glVertexAttribPointer(slot, element.count, element.type, element.normalized,
                            static_cast<GLsizei>(stride), reinterpret_cast<const void*>(offset));

      offset += vertex_layout::element_size(element.type) * element.count;
      slot++;
    }

    // Unbind to avoid leaking state.
    glBindVertexArray(0);

    // Save for later and return the VAO.
    cache.insert({key, vao});
    return vao;
  }

private:
  std::unordered_map<vao_key, GLuint, vao_key_hash> cache;
};

} // namespace nelo

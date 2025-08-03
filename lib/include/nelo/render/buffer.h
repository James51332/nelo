#pragma once

#include <cstddef>
#include <glad/glad.h>
#include <memory>
#include <vector>

namespace nelo
{

// We want to use a standardized buffer class for two reasons. For one, it let's us switch to a
// different render API in the future. Alternatively, it lets us multithread without altering render
// code. We won't let buffers be resizeable. The biggest issue that we are going to face in getting
// these to work is vertex arrays. These are very necessary part of the OpenGL pipeline that we
// really don't want the user to have to deal with if possible. Unfortunately, rendering without
// them is not possible. We need an API that we can use to define the vertex layout per renderer.
// Should we take some ownership from the renderer, and allow each renderer to have only one shader.
// This removes some boilerplate, but might take away freedom.
class buffer
{
public:
  // Creates a buffer with specified type, size (in bytes), and usage.
  buffer(GLenum type, std::size_t size, GLenum usage = GL_DYNAMIC_DRAW);
  ~buffer();

  void set_bytes(std::size_t offset, std::size_t count, const void* bytes);

  const GLuint object() const { return bufferID; }
  const GLenum buffer_type() const { return type; }
  const GLenum buffer_usage() const { return usage; }

private:
  GLuint bufferID;
  std::size_t size;
  GLenum type, usage;
};

// We recommend that classes use a buffer pool in place of a single buffer in case multiple buffers
// are needed. This lets us easily grow our memory pool if we run out of space.
class buffer_pool
{
public:
  // We can create a default pool with a size of 1 MB.
  buffer_pool(GLenum type = GL_ARRAY_BUFFER, std::size_t size = 1000000,
              GLenum usage = GL_DYNAMIC_DRAW);

  // Acquires a new vertex buffer.
  std::shared_ptr<buffer> acquire();

  // Resets the pool and allows for buffers to be acquired again.
  void reset();

private:
  std::vector<std::shared_ptr<buffer>> pool;
  std::size_t index = 0;

  std::size_t size;
  GLenum type, usage;
};

} // namespace nelo

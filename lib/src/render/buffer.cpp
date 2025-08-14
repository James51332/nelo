#include "render/buffer.h"

#include <format>
#include <stdexcept>

namespace nelo
{

buffer::buffer(GLenum type, std::size_t num_bytes, GLenum usage)
  : type(type), size(num_bytes), usage(usage)
{
  // We can create all buffers as empty. Almost all access to buffers will be after creation
  // anyways.
  glGenBuffers(1, &bufferID);
  glBindBuffer(type, bufferID);
  glBufferData(type, num_bytes, nullptr, usage);
  glBindBuffer(type, 0);
}

buffer::~buffer()
{
  glDeleteBuffers(1, &bufferID);
}

void buffer::set_bytes(std::size_t offset, std::size_t num_bytes, const void* bytes)
{
  // For now, there's not a great reason to unbind buffers. Likely is wasteful.
  if (offset + num_bytes > size)
  {
    auto msg =
        std::format("Invalid range to set buffer! (offset: {}, range: {})", offset, num_bytes);
    throw std::runtime_error(msg);
  }

  glBindBuffer(type, bufferID);
  glBufferSubData(type, offset, num_bytes, bytes);
}

buffer_pool::buffer_pool(GLenum type, std::size_t num_bytes, GLenum usage)
  : type(type), size(num_bytes), usage(usage)
{
  // Start by adding a buffer to the pool.
  acquire();
  reset();
}

std::shared_ptr<buffer> buffer_pool::acquire()
{
  while (index >= pool.size())
    pool.push_back(std::make_shared<buffer>(type, size, usage));

  index++;
  return pool[index - 1];
}

void buffer_pool::reset()
{
  index = 0;
}

} // namespace nelo

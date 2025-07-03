#pragma once

#include <cstdint>
#include <glad/glad.h>

namespace nelo
{

// We need a way to render to an image of any size. This way, we can set the size from our scene.
class framebuffer
{
public:
  framebuffer(std::uint32_t width, std::uint32_t height);
  ~framebuffer();

  // These resources shouldn't be copied.
  framebuffer(const framebuffer&) = delete;
  framebuffer& operator=(const framebuffer&) = delete;

  // We can still let them be moved.
  framebuffer(framebuffer&& other);
  framebuffer& operator=(framebuffer&& other);

  // Bind these for rendering.
  void bind();
  void unbind();

  // Getters
  std::uint32_t width() const { return fb_width; }
  std::uint32_t height() const { return fb_height; }

private:
  // TODO Consider if we want a front and back texture.
  GLuint fbo, color_texture, depth_texture;

  // We'll store these as well.
  std::uint32_t fb_width, fb_height;
};

} // namespace nelo

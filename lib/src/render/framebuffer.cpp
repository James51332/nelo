#include "render/framebuffer.h"

#include <format>
#include <stdexcept>
#include <utility>

namespace nelo
{

framebuffer::framebuffer(std::uint32_t width, std::uint32_t height)
{
  if (width < 0 || height < 0)
  {
    auto msg = std::format("Cannot create framebuffer with dimensions {} x {}", width, height);
    throw std::runtime_error(msg);
  }

  // Generate our framebuffer with some default configuration. We may want to adjust these later.
  glGenFramebuffers(1, &fbo);
  glBindFramebuffer(GL_FRAMEBUFFER, fbo);

  // Attach color buffer.
  glGenTextures(1, &color_texture);
  glBindTexture(GL_TEXTURE_2D, color_texture);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
  glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, color_texture, 0);

  // Attach depth buffer.
  glGenTextures(1, &depth_texture);
  glBindTexture(GL_TEXTURE_2D, depth_texture);
  glTexImage2D(GL_TEXTURE_2D, 0, GL_DEPTH_COMPONENT, width, height, 0, GL_DEPTH_COMPONENT, GL_FLOAT,
               nullptr);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
  glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, depth_texture, 0);

  // Let's make sure that our framebuffer is complete.
  if (glCheckFramebufferStatus(GL_FRAMEBUFFER) != GL_FRAMEBUFFER_COMPLETE)
  {
    auto msg = std::format("Framebuffer was not created due to OpenGL Error: 0x{:x}", glGetError());
    throw std::runtime_error(msg);
  }

  unbind();
}

framebuffer::~framebuffer()
{
  // Delete this object yo.
  glDeleteTextures(1, &color_texture);
  glDeleteTextures(1, &depth_texture);
  glDeleteFramebuffers(1, &fbo);
}

framebuffer::framebuffer(framebuffer&& other)
{
  // We can just use move assignment operator
  (*this) = std::move(other);
}

framebuffer& framebuffer::operator=(framebuffer&& other)
{
  // Steal all of the resources from the other.
  this->fbo = other.fbo;
  this->fb_width = other.fb_width;
  this->fb_height = other.fb_height;

  return (*this);
}

void framebuffer::bind()
{
  glBindFramebuffer(GL_FRAMEBUFFER, fbo);
  glDrawBuffer(GL_COLOR_ATTACHMENT0);
}

void framebuffer::unbind()
{
  glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

} // namespace nelo

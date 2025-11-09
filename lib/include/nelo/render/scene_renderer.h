#pragma once

#include <cstddef>
#include <memory>
#include <vector>

#include "anim/color.h"
#include "render/vao_cache.h"

namespace nelo
{

// Forward declare the scene and renderer classes to avoid circular includes.
class scene;
class renderer;

// The scene renderer is a special renderer that owns all other renderers. This handles the binding,
// sorting, and uniforms for all other renderers. As such, it does not inherit from the renderer
// class. Otherwise, it could be added to itself.
class scene_renderer
{
public:
  scene_renderer(std::uint32_t width = 1920, std::uint32_t height = 1080);

  void add_renderer(std::shared_ptr<renderer> new_renderer);
  void clear_renderers();

  void set_size(std::uint32_t width, std::uint32_t height);

  // Renders the scene and encodes. scene::play() is planned to be a wrapper around this method, so
  // that scene can focus on ECS logic.
  void play(scene& state, double start, double end);

private:
  // TODO These are some constants we want to eventually be configurable.
  constexpr static int fps = 60;
  constexpr static double scene_height = 5.0;
  inline constexpr static const char* const file_ext = ".mov";
  constexpr static color clear_color = glm::vec4(0.15f, 0.2f, 0.25f, 1.0f);

  void play_frame(scene& state, double t);

private:
  // For objects rendered at the same z-index, priority is given to the first renderer added.
  std::vector<std::shared_ptr<renderer>> renderers;

  std::uint32_t width, height;

  // OpenGL requires VAOs, but we abstract from the user via this API.
  vao_cache cache;
};

} // namespace nelo

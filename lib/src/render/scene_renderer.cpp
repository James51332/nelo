#include "render/scene_renderer.h"

#include <glad/glad.h>
#include <set>

#include "core/encoder.h"
#include "core/scene.h"
#include "render/command.h"
#include "render/framebuffer.h"
#include "render/renderer.h"

namespace nelo
{

scene_renderer::scene_renderer(std::uint32_t width, std::uint32_t height)
  : width(width), height(height)
{
}

void scene_renderer::add_renderer(std::shared_ptr<renderer> new_renderer)
{
  renderers.push_back(new_renderer);
}

void scene_renderer::clear_renderers()
{
  renderers.clear();
}

void scene_renderer::set_size(std::uint32_t width, std::uint32_t height)
{
  this->width = width;
  this->height = height;
}

void scene_renderer::play(scene& state, double start, double end)
{
  // Let's create a framebuffer with the scene width and height.
  framebuffer image(width, height);
  image.bind();

  // We create an encoder to output this to video.
  std::filesystem::path output_path = state.get_name() + file_ext;
  encoder encoder(static_cast<int>(width), static_cast<int>(height), fps, output_path);

  // Update the window until we are closed.
  double time = start;
  while (time <= end)
  {
    // Update the time.
    time += 1 / static_cast<double>(fps);

    // Play a single frame.
    play_frame(state, time);

    // Submit this frame to the encoder.
    encoder.submit();
  }

  // Complete the video write.
  encoder.end();
  image.unbind();
}

void scene_renderer::play_frame(scene& state, double t)
{
  // Clear the screen.
  glViewport(0, 0, width, height);
  glClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
  glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

  // Build a command buffer. We'll then iterate over our renderers and encode commands.
  command_buffer buffer;
  for (auto renderer : renderers)
    renderer->generate_commands(buffer, state, t);

  // Now, we need to friggin sort over these commands.
  std::vector<draw_command> commands = std::move(buffer.commands());
  std::sort(commands.begin(), commands.end(),
            [](auto& a, auto& b) { return a.z_index < b.z_index; });

  // Finally, we can actually call the draw commands.
  std::set<GLuint> usedShaders;

  for (auto& command : commands)
  {
    // First, we set our shader and ensure that we have the correct uniforms bound.
    GLuint shader = command.shader;
    glUseProgram(shader);

    if (!usedShaders.contains(shader))
    {
      glUniform2f(glGetUniformLocation(shader, "viewport_size"), static_cast<float>(width),
                  static_cast<float>(height));
      glUniform1f(glGetUniformLocation(shader, "scene_height"), scene_height);
      usedShaders.insert(shader);
    }

    // Next, we fetch the vao for our draw_command.
    auto& vbo = command.vbo;
    auto& ibo = command.ibo;
    auto& layout = command.layout;
    GLuint vao = cache.get_or_create(vbo, ibo, shader, layout);

    // Finally, we bind our vao and index buffers and submit!
    glBindVertexArray(vao);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, ibo->object());
    glDrawElements(command.primitive_type, command.index_count, command.index_type,
                   reinterpret_cast<const void*>(command.index_offset));
  }
}

} // namespace nelo

#pragma once

#include <cstddef>
#include <glad/glad.h>
#include <vector>

#include "render/buffer.h"
#include "render/layout.h"

namespace nelo
{

// This struct is used to allow renderers to submit commands ahead of time, which are then sorted
// and renderer back to front. Since each draw command has its own z-index, renderers can still
// batch, but only entities with the same z-index. We will adjust these objects as needed in the
// future so that we can multithread the renderer implicitly (build several frame command buffers
// simultaneously). The current feature which is most lacking is uniform data. We'll standardize
// this for now, but we will quite likely want uniform data to be customizable.
struct draw_command
{
  // This defines the pipeline. We'll make changes to these data types as we move to a more modern
  // render pipeline. However, I haven't decided which yet. Maybe nvrhi or SDL_gpu.
  GLuint shader;
  vertex_layout layout;
  GLenum primitive_type = GL_TRIANGLES;

  // These are the buffers we'll use. We may want to support multiple-stream encoding in the future.
  std::shared_ptr<buffer> vbo;
  std::shared_ptr<buffer> ibo;

  // Draw call information.
  std::uint32_t index_offset = 0;
  GLenum index_type = GL_UNSIGNED_INT;
  std::uint32_t index_count;

  // Use for sorting draw-commands.
  double z_index = 0.0;
};

// Eventually, this may become a first class API that we use. For now, we just use it as the
// submission system to avoid copying draw_commands from the renderer.
class command_buffer
{
public:
  void submit(draw_command cmd) { commands_list.push_back(cmd); }

  // We only need to support const access to this class.
  const std::vector<draw_command>& commands() const { return commands_list; }
  std::vector<draw_command>::const_iterator cbegin() const { return commands_list.cbegin(); }
  std::vector<draw_command>::const_iterator cend() const { return commands_list.cend(); }

private:
  std::vector<draw_command> commands_list;
};

} // namespace nelo

#pragma once

#include <functional>
#include <glad/glad.h>
#include <glm/glm.hpp>
#include <vector>

#include "types/curve.h"
#include "types/transform.h"

namespace nelo
{

// The complexity of curve rendering warrants a separate renderer. TODO As we have multiple
// renderers, we may want to abstract some of the graphics API logic.
class curve_renderer
{
public:
  struct curve_vertex
  {
    glm::vec3 position;
    glm::vec4 stroke;
    double weight;
    double alpha;
  };

public:
  curve_renderer(float scene_height = 5.0f);
  ~curve_renderer();

  void begin(double t);
  void end();

  // The renderer is not technically batched, but we'll stick to the same API.
  void submit(const transform& trans, const curve& curve);

  // This method takes a parametric path and converts it into a list of points which can be
  // rendered. For now, we are returning a std::vector of points, which may not be optimal, but I
  // think that it is just fine right now.
  std::vector<curve_vertex> subdivide(const curve& curve, double t);

private:
  // Private methods which help with generating geometry.
  void subdivide_helper(std::function<curve_vertex(double)>& sample,
                        std::vector<curve_vertex>& output, int min_index, int max_index,
                        double min_time, double max_time, int min_sub, int max_sub,
                        double threshold);

private:
  bool is_recording = false;
  double cur_time = 0.0;

  // The actual window height of the renderer.
  float scene_height = 0.0f;

  // The renderer is not batched, but we can allow the driver to not wait for the draw call before
  // moving to next curve by writing to a different part of the vbo. We can allocate ~5 megabytes
  // for storage.
  constexpr static std::uint32_t max_vertices = 100000;
  constexpr static std::uint32_t max_indices = 100000;
  std::uint32_t current_vertex = 0;
  std::uint32_t current_index = 0;
  curve_vertex* vertices = nullptr;
  std::uint32_t* indices = nullptr;

  // These are our gl contstructs which we use for rendering.
  GLuint curve_program;
  GLuint curve_vao;
  GLuint curve_vbo;
  GLuint curve_ibo;
};

} // namespace nelo

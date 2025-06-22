#include "render/curve_renderer.h"

#include <stdexcept>

#include "core/context.h"
#include "render/shaders.h"

namespace nelo
{

curve_renderer::curve_renderer(float scene_height)
  : scene_height(scene_height)
{
  // Load our shader program.
  curve_program = shaders::load("curve_vertex.glsl", "curve_fragment.glsl");

  // Now, we gotta do some friggin maneuvering to create a vao.
  glGenVertexArrays(1, &curve_vao);
  glBindVertexArray(curve_vao);

  // Create the vertex buffer and whatnot.
  vertices = new curve_vertex[max_vertices];
  glGenBuffers(1, &curve_vbo);
  glBindBuffer(GL_ARRAY_BUFFER, curve_vbo);
  glBufferData(GL_ARRAY_BUFFER, sizeof(curve_vertex) * max_vertices, nullptr, GL_DYNAMIC_DRAW);

  // Create the index buffer and whatnot. It can be statically generated.
  indices = new std::uint32_t[max_indices];

  // Generate the ibo (it will be attached to the vao).
  glGenBuffers(1, &curve_ibo);
  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, curve_ibo);
  glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(std::uint32_t) * max_indices, nullptr,
               GL_DYNAMIC_DRAW);

  // Let's setup our vertex attribs so that the shader doesn't throw a friggin fit.
  glEnableVertexAttribArray(0);
  glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(curve_vertex), (void*)0);
  glEnableVertexAttribArray(1);
  glVertexAttribPointer(1, 4, GL_FLOAT, GL_FALSE, sizeof(curve_vertex), (void*)sizeof(glm::vec3));
  glEnableVertexAttribArray(2);
  glVertexAttribPointer(2, 1, GL_DOUBLE, GL_FALSE, sizeof(curve_vertex),
                        (void*)(sizeof(glm::vec3) + sizeof(glm::vec4)));
  glVertexAttribPointer(3, 1, GL_DOUBLE, GL_FALSE, sizeof(curve_vertex),
                        (void*)(sizeof(glm::vec3) + sizeof(glm::vec4) + sizeof(double)));
}

curve_renderer::~curve_renderer()
{
  delete[] vertices;
  delete[] indices;
  glDeleteProgram(curve_program);
  glDeleteBuffers(1, &curve_vbo);
  glDeleteBuffers(1, &curve_ibo);
  glDeleteVertexArrays(1, &curve_vao);
}

void curve_renderer::begin(double t)
{
  if (is_recording)
    throw std::runtime_error("Cannot begin curve renderer because recording has begun!");

  is_recording = true;
  cur_time = t;

  current_vertex = 0;
  current_index = 0;

  glUseProgram(curve_program);
  glUniform2f(glGetUniformLocation(curve_program, "viewport_size"), context::size().x,
              context::size().y);
  glUniform1f(glGetUniformLocation(curve_program, "scene_height"), scene_height);
}

void curve_renderer::end()
{
  if (!is_recording)
    throw std::runtime_error("Cannot end curve renderer because recording has not begun!");

  is_recording = false;
}

void curve_renderer::submit(const transform& trans, const curve& curve)
{
  if (!is_recording)
    throw std::runtime_error("Cannot submit curve to render because recording has not begun!");

  // The process for generating geometry is involved. We start by generating a list of points
  // which should smoothly capture the path when rendered as a polyline. Then we generate a smooth
  // set of triangles with the desired thickness around the polyline.
  std::vector<curve_vertex> points = subdivide(curve, cur_time);

  // Next, we submit the set of points to the renderer to encode into the vertex and index buffers
  // respectively. How do we do this? Here's the basic idea. For each vertex two vertices, we'll
  // create a trapezoid between.
  std::uint32_t start_vertex = current_vertex;
  std::uint32_t start_index = current_index;

  for (int i = 0; i < points.size() - 1; i++)
  {
    curve_vertex a = points[i];
    curve_vertex b = points[i + 1];

    // Only considering normal for this one.
    glm::vec3 dir = glm::normalize(b.position - a.position);
    glm::vec3 normal = glm::vec3(-dir.y, dir.x, 0.0f);

    glm::vec3 a1 = a.position + normal * static_cast<float>(a.weight);
    glm::vec3 a2 = a.position - normal * static_cast<float>(a.weight);
    glm::vec3 b1 = b.position + normal * static_cast<float>(b.weight);
    glm::vec3 b2 = b.position - normal * static_cast<float>(b.weight);

    // Add vertices
    vertices[current_vertex + 0] = {a1, a.stroke, a.weight, a.alpha};
    vertices[current_vertex + 1] = {b1, b.stroke, b.weight, b.alpha};
    vertices[current_vertex + 2] = {b2, b.stroke, b.weight, b.alpha};
    vertices[current_vertex + 3] = {a2, a.stroke, b.weight, a.alpha};

    // Add indices (2 triangles)
    indices[current_index + 0] = current_vertex + 0;
    indices[current_index + 1] = current_vertex + 1;
    indices[current_index + 2] = current_vertex + 2;
    indices[current_index + 3] = current_vertex + 0;
    indices[current_index + 4] = current_vertex + 2;
    indices[current_index + 5] = current_vertex + 3;

    current_vertex += 4;
    current_index += 6;
  }

  // Now we can copy our geometry to the buffers and render it.
  glBindVertexArray(curve_vao);
  glBindBuffer(GL_ARRAY_BUFFER, curve_vbo);
  glBufferSubData(GL_ARRAY_BUFFER, start_vertex * sizeof(curve_vertex),
                  (current_vertex - start_vertex) * sizeof(curve_vertex), vertices + start_vertex);

  glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, curve_ibo);
  glBufferSubData(GL_ELEMENT_ARRAY_BUFFER, start_index * sizeof(std::uint32_t),
                  (current_index - start_index) * sizeof(std::uint32_t), indices + start_index);

  // Bind our vao and program and render that curve.
  glUseProgram(curve_program);
  glDrawElements(GL_TRIANGLES, current_index - start_index, GL_UNSIGNED_INT, nullptr);
}

std::vector<curve_renderer::curve_vertex> curve_renderer::subdivide(const curve& curve, double t)
{
  // Some details for how detailed we want to be.
  int min_sub = curve.min_subdivisions.sample(t);
  int max_sub = curve.max_subdivisions.sample(t);
  double threshold = curve.threshold.sample(t);

  // We'll use recursion to subdivide wherever the path is deemed curvy.
  double start = curve.start.sample(t);
  double end = curve.end.sample(t);

  // If end is less than start or zero, then we render the length of the path.
  if (end < start || end == 0.0)
  {
    start = 0.0;
    end = curve.spline.length();
  }

  // Create a shorthand lambda for evaluating the curve at a given alpha.
  std::function<curve_vertex(double)> sample = [&curve, t](double alpha) -> curve_vertex
  {
    return curve_vertex{.position = curve.spline.sample(t).sample(alpha),
                        .stroke = curve.stroke.sample(t).sample(alpha),
                        .weight = curve.weight.sample(t).sample(alpha),
                        .alpha = alpha};
  };

  // TODO Investigate if std::list gives better insertion performance if needed.
  std::vector<curve_vertex> output = {sample(start), sample(end)};

  // Call the recursive helper to perform our work for us.
  subdivide_helper(sample, output, 0, 1, start, end, min_sub, max_sub, threshold);

  // Return our final computed list of points.
  return std::move(output);
}

void curve_renderer::subdivide_helper(std::function<curve_vertex(double)>& sample,
                                      std::vector<curve_vertex>& output, int min_index,
                                      int max_index, double min_time, double max_time, int min_sub,
                                      int max_sub, double threshold)
{
  // If this is greater than zero, we must subdivide. Skip the computation.
  bool subdivide = min_sub > 0;

  // We cannot subdivide if we have reach subdivision limit.
  if (max_sub == 0)
    return;

  // We'll save these values from the subdivision check.
  double mid_time = (min_time + max_time) * 0.5;
  curve_vertex mid_point = sample(mid_time);

  // Check if we need to subdivide using the mathy check. We need to compute the distance from the
  // start and end point to the line. Here's how we'll do it. First, compute the direction vector
  // from the start to end_point. Next compute the displacement vector from the start to the
  // midpoint. Finally, compute the magnitude their cross product to get the distance going in the
  // perpendincular direction. We only do this if we don't know if we need to subdivide. Min
  // subdivsions is 8 by default so we should be bypassing this as often as possible.
  if (!subdivide)
  {
    glm::vec3 start_point = output[min_index].position;
    glm::vec3 end_point = output[max_index].position;
    glm::vec3 direction = glm::normalize(end_point - start_point);
    glm::vec3 displacement = mid_point.position - start_point;
    float distance = glm::length(glm::cross(direction, displacement));
    subdivide = (distance >= static_cast<float>(threshold));
  }

  // Our base case is when we do not need to subdivide.
  if (subdivide)
  {
    // Insert the parent first, since we need the indices to change.
    output.insert(output.begin() + max_index, mid_point);

    // One thing that is tricky is indices into the array get changed after we insert, so we have
    // to be clever. The solution is to subdivide from back to front.
    subdivide_helper(sample, output, max_index, max_index + 1, mid_time, max_time, min_sub - 1,
                     max_sub - 1, threshold);
    subdivide_helper(sample, output, min_index, min_index + 1, min_time, mid_time, min_sub - 1,
                     max_sub - 1, threshold);
  }
}

} // namespace nelo

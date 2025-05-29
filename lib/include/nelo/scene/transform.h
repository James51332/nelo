#pragma once

#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

namespace nelo
{

// All objects in the scene will have a transform component. We allow transforms
// to be composed in one of two ways. The positions can be added, or they can be
// composed as matrices. We are going to define a transform in three dimensions
// to support those as they arrive, and create a nice API that allows users to
// ignore the definition if they choose.
struct transform
{
  // Constructors
  transform(const glm::vec3& pos = glm::vec3(1.0f), const glm::vec3& scl = glm::vec3(1.0f),
            const glm::quat& rot = glm::quat(1.0f, 0.0f, 0.0f, 0.0f))
    : position(pos), scale(scl), rotation(rot)
  {
  }

  // Setter API
  void set_position(const glm::vec2& pos) { set_position(glm::vec3(pos, 0.0f)); }
  void set_position(const glm::vec3& pos)
  {
    matrix_dirty = true;
    position = pos;
  }

  void set_scale(const glm::vec3& scl)
  {
    matrix_dirty = true;
    scale = scl;
  }

  void set_rotation(const glm::quat& rot)
  {
    matrix_dirty = true;
    rotation = rot;
  }

  // Getter API
  const glm::vec3& get_position() const { return position; }
  const glm::vec3& get_scale() const { return scale; }
  const glm::quat& get_rotation() const { return rotation; }

private:
private:
  // We want to cache whenever possible, so we require these values to be updated via setters. These
  // are the source of truth for this struct.
  glm::vec3 position = glm::vec3(0.0f);
  glm::vec3 scale = glm::vec3(1.0f);
  glm::quat rotation = glm::quat(1.0f, 0.0f, 0.0f, 0.0f);

  // We'll cache the matrix so that we don't have to create. We compute lazily.
  bool matrix_dirty = true;
  glm::mat4 cached_matrix = glm::mat4(1.0f);
};

} // namespace nelo

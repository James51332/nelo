#pragma once

#include <glad/glad.h>
#include <stdexcept>
#include <string>
#include <vector>

namespace nelo
{

// A vertex layout defines the way that a vertex buffer is laid out. For now, we do not support
// multi-stream rendering, so we can only submit draw calls from one VBO.
struct vertex_layout
{
  struct element
  {
    std::string name;
    GLenum type;
    GLint count;
    GLboolean normalized = false;

    bool operator==(const element& other) const
    {
      return name == other.name && type == other.type && count == other.count &&
             normalized == other.normalized;
    }
  };

  std::vector<element> elements;

  vertex_layout()
    : elements()
  {
  }

  vertex_layout(std::initializer_list<element> elements)
    : elements(elements)
  {
  }

  // Maps the supported enums for glVertexAttribPointer to their respective size in bytes.
  // https://registry.khronos.org/OpenGL-Refpages/gl4/html/glVertexAttribPointer.xhtml
  inline static std::size_t element_size(GLenum type)
  {
    switch (type)
    {
      case GL_BYTE: return 1;
      case GL_UNSIGNED_BYTE: return 1;
      case GL_SHORT: return 2;
      case GL_UNSIGNED_SHORT: return 2;
      case GL_INT: return 4;
      case GL_HALF_FLOAT: return 2;
      case GL_FLOAT: return 4;
      case GL_DOUBLE: return 8;
      case GL_FIXED: return 4;
      case GL_INT_2_10_10_10_REV: return 4;
      case GL_UNSIGNED_INT_2_10_10_10_REV: return 4;
      case GL_UNSIGNED_INT_10F_11F_11F_REV: return 4;

      default: throw std::runtime_error("Unsupported type for vertex_layout!");
    }
  }

  bool operator==(const vertex_layout& other) const { return elements == other.elements; }
};

} // namespace nelo

// We'll inject a hash for vertex_layout into standard namespace.
namespace std
{

template <>
struct hash<nelo::vertex_layout::element>
{
  std::size_t operator()(const nelo::vertex_layout::element& e) const
  {
    std::size_t h1 = std::hash<std::string>()(e.name);
    std::size_t h2 = std::hash<GLenum>()(e.type);
    std::size_t h3 = std::hash<GLint>()(e.count);
    std::size_t h4 = std::hash<GLboolean>()(e.normalized);
    return (((h1 ^ (h2 << 1)) >> 1) ^ (h3 << 1)) ^ (h4 << 2);
  }
};

template <>
struct hash<nelo::vertex_layout>
{
  std::size_t operator()(const nelo::vertex_layout& layout) const
  {
    std::size_t seed = 0;
    for (const auto& el : layout.elements)
    {
      seed ^= hash<nelo::vertex_layout::element>()(el) + 0x9e3779b9 + (seed << 6) + (seed >> 2);
    }
    return seed;
  }
};

} // namespace std

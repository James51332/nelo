#pragma once

#include <filesystem>
#include <glad/glad.h>

namespace nelo
{

// Shaders in nelo are loaded from a file whenever possible, but core shaders are also cached so
// that the binary doesn't need to be shipped with them. This class is an abstraction over this
// caching, so we can just ask for a shader by filename, and ignore the implementation.
class shaders
{
public:
  // Attempts to load a shader from a file. If the file is not found, attempts to load from shader
  // cache. If both fail, or the shader cannot be compiled, throws a runtime error. For now, we have
  // to load vertex and fragment shaders separately, but we may support some form of preprocessing
  // in the future.
  static GLuint load(const std::filesystem::path& vertex, const std::filesystem::path& fragment);

private:
  static std::string read_file(const std::filesystem::path& file);
  static GLuint compile(GLint shader_type, const std::filesystem::path& source);
};

} // namespace nelo

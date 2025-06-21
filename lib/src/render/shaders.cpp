#include "render/shaders.h"

#include <format>
#include <fstream>
#include <glad/glad.h>
#include <stdexcept>

#include "core/log.h"
#include "embedded_shaders.h"

namespace nelo
{

GLuint shaders::load(const std::filesystem::path& vertex, const std::filesystem::path& fragment)
{
  // Compile the shaders.
  GLuint vert_shader = compile(GL_VERTEX_SHADER, vertex);
  GLuint frag_shader = compile(GL_FRAGMENT_SHADER, fragment);

  // Create and link the shader program.
  GLuint program = glCreateProgram();
  glAttachShader(program, vert_shader);
  glAttachShader(program, frag_shader);
  glLinkProgram(program);

  // Check for success, and let the user know if something went wrong.
  int success;
  glGetProgramiv(program, GL_LINK_STATUS, &success);
  if (!success)
  {
    constexpr std::size_t buffer_size = 512;
    char buffer[buffer_size];

    // Print the link error.
    glGetProgramInfoLog(program, buffer_size, nullptr, buffer);
    log::out(buffer);

    // Throw a runtime exception.
    std::string msg = std::format("Unable to link {} and {}!", vertex.string(), fragment.string());
    throw std::runtime_error(msg);
  }

  return program;
}

std::string shaders::read_file(const std::filesystem::path& path)
{
  // We first look for the actual file, then look in the shaders directory.
  std::vector<std::filesystem::path> prefixes = {"", "shaders"};

  // Start by looking for the file.
  for (auto prefix : prefixes)
  {
    // If the file does not exist, we'll skip.
    std::filesystem::path prefixed_path = prefix / path;
    if (!std::filesystem::exists(prefixed_path))
      continue;

    std::ifstream file(prefixed_path);
    if (file.is_open())
    {
      auto size = std::filesystem::file_size(prefixed_path);
      std::string content(size, '\0');
      file.read(&content[0], size);
      file.close();

      return content;
    }
  }

  // Print out a message to the console that we are falling back to embedded shader files.
  std::string key = path.string();
  log::out("Unable to read {}. Looking for fallback in shader cache", key);

  // If we don't find it, look in the shader cache.
  if (embedded_shaders::fallback_shader_sources.contains(key))
    return embedded_shaders::fallback_shader_sources.at(key);

  // We can throw an error at this point.
  std::string error = std::format("Shader was unable to be found at {}", key);
  throw std::runtime_error(error);
}

GLuint shaders::compile(GLint shader_type, const std::filesystem::path& path)
{
  // Start by loading the file from disk.
  std::string source = read_file(path);
  const char* source_cstr = source.c_str();

  // Create our shader object and give it the source.
  GLuint shader = glCreateShader(shader_type);
  glShaderSource(shader, 1, &source_cstr, nullptr);
  glCompileShader(shader);

  int success;
  glGetShaderiv(shader, GL_COMPILE_STATUS, &success);
  if (!success)
  {
    constexpr std::size_t buffer_size = 512;
    char buffer[buffer_size];

    // Print the link error.
    glGetShaderInfoLog(shader, buffer_size, nullptr, buffer);
    log::out(buffer);

    // Throw a runtime exception.
    std::string msg = std::format("Unable to compile {}. See log for details!", path.string());
    throw std::runtime_error(msg);
  }

  // If we did succeed, we can return the shader!
  return shader;
}

} // namespace nelo

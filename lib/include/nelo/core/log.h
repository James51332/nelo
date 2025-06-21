#pragma once

#include <filesystem>
#include <format>
#include <fstream>
#include <mutex>
#include <string>

namespace nelo
{

// The nelo log provides a unified logging interface. This allows us to write to a file, or to just
// quiet the output when it is not verbose. We can also write to a file if we choose. By default,
// only console logging is enabled.
class log
{
public:
  static void use_console(bool use = true);
  static void use_file(const std::filesystem::path& file = "log.txt", bool use = true);

  static void out(const std::string& msg);

  // Wrapper around std::format to allow for formatting printing.
  template <typename... Args>
  static void out(std::format_string<Args...> fmt, Args&&... args)
  {
    if (!uses_console && !uses_file)
      return;

    out(std::format(fmt, std::forward<Args>(args)...));
  }

private:
  static bool uses_console;
  static std::mutex console_mutex;

  static bool uses_file;
  static std::mutex file_mutex;
  static std::filesystem::path file_path;
  static std::ofstream file_stream;
};

} // namespace nelo

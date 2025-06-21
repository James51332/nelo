#include "core/log.h"

#include <iostream>

namespace nelo
{

bool log::uses_console = true;
std::mutex log::console_mutex;

bool log::uses_file = false;
std::mutex log::file_mutex;
std::filesystem::path log::file_path;
std::ofstream log::file_stream;

void log::use_console(bool use)
{
  uses_console = use;
}

void log::use_file(const std::filesystem::path& path, bool use)
{
  // Close the existing file if there is one.
  if (file_stream.is_open())
    file_stream.close();

  if (!use)
  {
    uses_file = false;
    return;
  }

  // Open our file.
  file_path = path;
  file_stream.open(path, std::ios::out | std::ios::trunc);
  if (file_stream.is_open())
    uses_file = true;
  else
    out("Unable to open log file: {}", file_path.string());
}

void log::out(const std::string& msg)
{
  if (!uses_console && !uses_file)
    return;

  if (uses_console)
  {
    std::lock_guard lock(console_mutex);
    std::cout << msg << std::endl;
  }

  if (uses_file)
  {
    std::lock_guard lock(file_mutex);
    file_stream << msg << std::endl;
  }
}

} // namespace nelo

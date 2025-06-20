# Nelo is not dependent on being shipped with shaders. Here is the system design. We try to load 
# from files whereever possible. Our fallback is that we cache the shaders directory at build time
# into the executable using this script, so the binary can be distrubuted alone (with ffmpeg dep.).
cmake_minimum_required(VERSION 3.24)

if(NOT DEFINED SHADER_DIR)
  message(FATAL_ERROR "SHADER_DIR not set")
endif()

if(NOT DEFINED OUTPUT_HEADER)
  message(FATAL_ERROR "OUTPUT_HEADER not set")
endif()

file(GLOB_RECURSE SHADER_FILES "${SHADER_DIR}/*.*")

set(GENERATED "// Auto-generated shader header\n#pragma once\n")
set(GENERATED "${GENERATED}#include <unordered_map>\n#include <string>\n\n")
set(GENERATED "${GENERATED}namespace embedded_shaders \n{\n\n")

set(MAP_ENTRIES "")
set(VARNAMES_SEEN "")

foreach(SHADER_FILE ${SHADER_FILES})
  file(RELATIVE_PATH REL_PATH ${SHADER_DIR} ${SHADER_FILE})
  
  # Create a safe variable name with hash
  string(REPLACE "/" "_" VAR_NAME ${REL_PATH})
  string(REPLACE "." "_" VAR_NAME ${VAR_NAME})
  string(MD5 HASH ${REL_PATH})
  string(SUBSTRING ${HASH} 0 8 HASH_SUFFIX)
  set(VAR_NAME "${VAR_NAME}_${HASH_SUFFIX}")

  # Read and escape shader contents
  file(READ ${SHADER_FILE} CONTENTS)
  string(REPLACE "\\" "\\\\" CONTENTS "${CONTENTS}")
  string(REPLACE "\"" "\\\"" CONTENTS "${CONTENTS}")
  string(REPLACE "\n" "\\n\"\n\"" CONTENTS "${CONTENTS}")

  # Write variable
  set(GENERATED "${GENERATED}inline const char* ${VAR_NAME} = \"${CONTENTS}\";\n\n")

  # Write map entry with original relative path
  set(MAP_ENTRIES "${MAP_ENTRIES}  {\"${REL_PATH}\", ${VAR_NAME}},\n")
endforeach()

# Finalize map
set(GENERATED "${GENERATED}inline const std::unordered_map<std::string, const char*> fallback_shader_sources = \n{\n${MAP_ENTRIES}};\n")
set(GENERATED "${GENERATED}\n} // namespace embedded_shaders\n")
file(WRITE ${OUTPUT_HEADER} "${GENERATED}")

# Set our cmake version
cmake_minimum_required(VERSION 3.24)

# Declare our project
project(nelo-api)

# Set the C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Load SDL project into build
set(SDL_STATIC ON)
set(SDL_TESTS OFF)
set(SDL_TEST_LIBRARY OFF)
set(SDL_DISABLE_INSTALL ON)
set(SDL_SHARED OFF)
add_subdirectory(thirdparty/sdl)

# We also use glad to load our opengl
add_subdirectory(thirdparty/glad)

# glm is our math library for fast compiler intrinsics.
add_subdirectory(thirdparty/glm)

# For lua, we have a light wrapper to build the official c. We use sol2 as a wrapper to interface with lua.
add_subdirectory(thirdparty/lua)
add_subdirectory(thirdparty/sol2)

# ffmpeg has weird license restrictions, so we won't ship it with this app, we'll just mark it as a dependency.
find_path(AVCODEC_INCLUDE_DIR libavcodec/avcodec.h REQUIRED)
find_library(AVCODEC_LIBRARY avcodec REQUIRED)
find_path(AVUTIL_INCLUDE_DIR libavutil/avutil.h REQUIRED)
find_library(AVUTIL_LIBRARY avutil REQUIRED)
find_path(AVFORMAT_INCLUDE_DIR libavformat/avformat.h REQUIRED)
find_library(AVFORMAT_LIBRARY avformat REQUIRED)
find_path(SWSCALE_INCLUDE_DIR libswscale/swscale.h REQUIRED)
find_library(SWSCALE_LIBRARY swscale REQUIRED)

# Source files
set(SRC_FILES 
  src/core/context.cpp
  src/core/encoder.cpp
  src/core/log.cpp
  src/core/scene.cpp
  src/render/circle_renderer.cpp
  src/render/curve_renderer.cpp
  src/render/shaders.cpp
  src/script/lua_timeline.cpp
  src/script/lua_types.cpp
  src/script/server.cpp)

# Header files
set(HEADER_FILES
  include/nelo/anim/color.h
  include/nelo/anim/easing.h
  include/nelo/anim/timeline.h
  include/nelo/anim/keyframe.h
  include/nelo/anim/traits.h
  include/nelo/core/collection.h
  include/nelo/core/context.h
  include/nelo/core/entity.h
  include/nelo/core/log.h
  include/nelo/core/scene.h
  include/nelo/render/circle_renderer.h
  include/nelo/render/curve_renderer.h
  include/nelo/render/shaders.h
  include/nelo/script/lua_timeline.h
  include/nelo/script/lua_types.h
  include/nelo/script/server.h
  include/nelo/types/curve.h
  include/nelo/types/path.h
  include/nelo/types/shapes.h
  include/nelo/types/transform.h)

# Define the executable for the program
add_library(nelo-api STATIC ${SRC_FILES} ${HEADER_FILES})

# For shaders, we will build our cache using the shader function
set(EMBEDDED_HEADER "${CMAKE_BINARY_DIR}/cache/embedded_shaders.h")
set(SHADER_DIR "${CMAKE_SOURCE_DIR}/shaders")
file(GLOB_RECURSE SHADER_FILES "${SHADER_DIR}/*.*")

add_custom_command(
    OUTPUT ${EMBEDDED_HEADER}
    COMMAND ${CMAKE_COMMAND} 
      -DSHADER_DIR=${SHADER_DIR}
      -DOUTPUT_HEADER=${EMBEDDED_HEADER} 
      -P ${CMAKE_SOURCE_DIR}/cmake/EmbedShaders.cmake
    DEPENDS ${SHADER_FILES}
    COMMENT "Regenerating embedded shaders..."
)
add_custom_target(generate_shaders DEPENDS ${EMBEDDED_HEADER})
add_dependencies(nelo-api generate_shaders)

# Define the include directory to begin within the source directory
target_include_directories(nelo-api
  PUBLIC
    include/nelo
  PRIVATE
    "${CMAKE_BINARY_DIR}/cache")

# For apps the implement nelo, we add the include directory as a system dependency
target_include_directories(nelo-api
  PUBLIC SYSTEM
    include
    ${AVCODEC_INCLUDE_DIR}
    ${AVUTIL_INCLUDE_DIR}
    ${AVFORMAT_INCLUDE_DIR}
    ${SWSCALE_INCLUDE_DIR})

# Add our dependencies
target_link_libraries(nelo-api
  PUBLIC
    SDL3::SDL3-static
    glad
    glm::glm
    ${AVCODEC_LIBRARY}
    ${AVUTIL_LIBRARY}
    ${AVFORMAT_LIBRARY}
    ${SWSCALE_LIBRARY}
    lua
    sol2::sol2)

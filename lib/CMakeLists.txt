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
  src/renderer/context.cpp
  src/renderer/renderer.cpp
  src/video/encoder.cpp
  src/scene/scene.cpp)

# Header files
set(HEADER_FILES
  include/nelo/scene/timeline.h
  include/nelo/scene/keyframe.h
  include/nelo/scene/traits.h
  include/nelo/scene/easing.h
  include/nelo/scene/entity.h
  include/nelo/scene/collection.h
  include/nelo/scene/scene.h)

# Define the executable for the program
add_library(nelo-api STATIC ${SRC_FILES} ${HEADER_FILES})

# Define the include directory to begin within the source directory
target_include_directories(nelo-api
  PUBLIC
    include/nelo)

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
    ${SWSCALE_LIBRARY})

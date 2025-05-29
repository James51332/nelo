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

# Source files
set(SRC_FILES 
  src/renderer/context.cpp
  src/renderer/renderer.cpp)

# Header files
set(HEADER_FILES
  include/nelo/scene/timeline.h
  include/nelo/scene/keyframe.h
  include/nelo/scene/traits.h
  include/nelo/scene/easing.h)

# Define the executable for the program
add_library(nelo-api STATIC ${SRC_FILES} ${HEADER_FILES})

# Define the include directory to begin within the source directory
target_include_directories(nelo-api
  PRIVATE
    include/nelo)

# For apps the implement nelo, we add the include directory as a system dependency
target_include_directories(nelo-api
  PUBLIC SYSTEM
    include)

# Add our dependencies
target_link_libraries(nelo-api
  PUBLIC
    SDL3::SDL3-static
    glad
    glm::glm)

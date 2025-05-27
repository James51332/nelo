# Set our cmake version
cmake_minimum_required(VERSION 3.20)

# Load SDL project into build
set(SDL_STATIC ON)
set(SDL_TESTS OFF)
set(SDL_TEST_LIBRARY OFF)
set(SDL_DISABLE_INSTALL ON)
set(SDL_SHARED OFF)
add_subdirectory(thirdparty/sdl)

# We also use glad to load our opengl
add_subdirectory(thirdparty/glad)

# Basic info about the project
project(nelo)

# Set the C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Source files
file(GLOB_RECURSE SRC_FILES CMAKE_CONFIGURE_DEPENDS "src/*.cpp" "src/*.h src/**.cpp src/**.h")

# Define the executable for the program
add_executable(nelo ${SRC_FILES})

# Define the include directory to begin within the source directory
target_include_directories(nelo
                             PRIVATE
                               src)

# Add our dependencies
target_link_libraries(nelo
                        PUBLIC
                          SDL3::SDL3-static
                          glad)

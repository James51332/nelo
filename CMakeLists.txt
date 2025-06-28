# Set our minimum required cmake version.
cmake_minimum_required(VERSION 3.24)

# Project
project(nelo)

# Set the C++ standard since we use modern language features. 
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Enable testing for the project
enable_testing()

# Include our other cmake files.
add_subdirectory(lib)
add_subdirectory(app)
add_subdirectory(module)
add_subdirectory(tests)

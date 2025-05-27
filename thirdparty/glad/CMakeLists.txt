# glad doesn't actually come with a CMakeLists.txt. This is just an 
# easy way to import into our project without adding it as a dependency.
cmake_minimum_required(VERSION 3.20)

project(glad)
add_library(glad STATIC src/glad.c)
target_include_directories(glad SYSTEM PUBLIC include)

cmake_minimum_required(VERSION 3.24)

project(nelo-app)

set(SRC_FILES
  src/main.cpp)

add_executable(nelo ${SRC_FILES})

target_link_libraries(nelo
  PRIVATE
    nelo-api ffmpeg)

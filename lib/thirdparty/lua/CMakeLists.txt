cmake_minimum_required(VERSION 3.24)

project(lua C)

file(GLOB_RECURSE LUA_SOURCES
    CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/lua/*.c"
)

add_library(lua STATIC ${LUA_SOURCES})

target_include_directories(lua PUBLIC SYSTEM ${CMAKE_CURRENT_SOURCE_DIR}/lua)

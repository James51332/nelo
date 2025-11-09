cmake_minimum_required(VERSION 3.24)

project(lua C)

file(GLOB_RECURSE LUA_SOURCES
    CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/lua/*.c"
)

# MSVC doesn't like duplicate main symbols. We'll filter out interpreters.
list(FILTER LUA_SOURCES EXCLUDE REGEX ".*/onelua\\.c$")
list(FILTER LUA_SOURCES EXCLUDE REGEX ".*/lua\\.c$")
list(FILTER LUA_SOURCES EXCLUDE REGEX ".*/luac\\.c$")

add_library(lua STATIC ${LUA_SOURCES})

target_include_directories(lua PUBLIC SYSTEM ${CMAKE_CURRENT_SOURCE_DIR}/lua)

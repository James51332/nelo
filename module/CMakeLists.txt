cmake_minimum_required(VERSION 3.24)
project(nelo-module)

# The only file for the module is this one. We build this separately because the main library is 
# build statically. Lua requires a dynamic library since it loads via dlopen.
add_library(nelo-module SHARED src/lua_module.cpp)

# Link against the main nelo library.
target_link_libraries(nelo-module
  PRIVATE
    nelo-api)

# Make sure the libray has the correct name
set_target_properties(nelo-module 
  PROPERTIES
    PREFIX ""
    OUTPUT_NAME "nelo"
)

# If we are on Windows, then build a .dll file. Otherwise, build a .so library.
if(WIN32)
  set_target_properties(nelo-module PROPERTIES SUFFIX ".dll")
else()
  set_target_properties(nelo-module PROPERTIES SUFFIX ".so")
  target_compile_options(nelo-module PRIVATE -fPIC)
  target_link_options(nelo-module PRIVATE -undefined dynamic_lookup)
endif()

# If we set LUA_INSTALL_DIR, then we can also configure an install command.
if(DEFINED LUA_INSTALL_DIR)
  install(TARGETS nelo-module 
    LIBRARY DESTINATION ${LUA_INSTALL_DIR}
    RUNTIME DESTINATION ${LUA_INSTALL_DIR}) # Needed for Windows
endif()

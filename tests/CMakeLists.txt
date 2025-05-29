# Set our minimum required cmake version.
cmake_minimum_required(VERSION 3.24)

# CTest requires us to enable testing for no reason.
enable_testing()

# Include CTest for testing
include(CTest)

# Set the project
project(nelo-tests)

# Generate a driver for our test file.
set(NeloTests
  scene/timeline.cpp
)
create_test_sourcelist(Tests nelo_tests.cpp ${NeloTests})

# Add the executable
add_executable(nelo-tests ${Tests})
target_link_libraries(nelo-tests PRIVATE nelo-api)

# # Add all the ADD_TEST for each test
foreach(test ${NeloTests})

  # Remove the file extension 
  cmake_path(REMOVE_EXTENSION test)

  # Add our test to our config
  add_test(NAME ${test} COMMAND nelo-tests ${test})
  
endforeach()

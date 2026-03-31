cmake_minimum_required (VERSION 3.17)

set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

project (model_description_validator)

# Set libxml2 root
set(LIBXML2_ROOT "E:/WS/Reference-FMUs/rust/libxml2/install" CACHE PATH "libxml2 installation root")

# Create both object library and static library
add_library(model_description_validator_objlib OBJECT
  src/md_valid.c
)

add_library(model_description_validator STATIC
  $<TARGET_OBJECTS:model_description_validator_objlib>
)

target_include_directories(model_description_validator_objlib PRIVATE
  "${LIBXML2_ROOT}/include/libxml2"
  include
)

target_include_directories(model_description_validator PRIVATE
  "${LIBXML2_ROOT}/include/libxml2"
  include
)

# Link libxml2 static library
find_library(LIBXML2_LIBRARY NAMES libxml2s PATHS "${LIBXML2_ROOT}/lib" NO_DEFAULT_PATH REQUIRED)
target_link_libraries(model_description_validator_objlib INTERFACE "${LIBXML2_LIBRARY}")
target_link_libraries(model_description_validator INTERFACE "${LIBXML2_LIBRARY}")

install(TARGETS model_description_validator
    DESTINATION ${CMAKE_SOURCE_DIR}/install
)

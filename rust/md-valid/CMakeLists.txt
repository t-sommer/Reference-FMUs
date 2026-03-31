cmake_minimum_required (VERSION 3.17)

set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

project (model_description_validator)

# Set libxml2 root
set(LIBXML2_ROOT "C:/Users/tsr2/Downloads/libxml2-v2.15.2/install" CACHE PATH "libxml2 installation root")

add_library(model_description_validator STATIC
  src/md_valid.c
)

target_include_directories(model_description_validator PRIVATE
  "${LIBXML2_ROOT}/include/libxml2"
  include
)

# Link libxml2 (DLL import library)
find_library(LIBXML2_LIBRARY NAMES libxml2 PATHS "${LIBXML2_ROOT}/lib" NO_DEFAULT_PATH REQUIRED)
target_link_libraries(model_description_validator PUBLIC "${LIBXML2_LIBRARY}")

install(TARGETS model_description_validator
    DESTINATION ${CMAKE_SOURCE_DIR}/install
)

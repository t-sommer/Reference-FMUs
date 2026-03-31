# Build libxml2

```
cmake -S libxml2 -B libxml2/build -D CMAKE_INSTALL_PREFIX=libxml2/install -D CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded -D LIBXML2_WITH_ICONV=NO -D BUILD_SHARED_LIBS=NO -D LIBXML2_WITH_PROGRAMS=NO -D LIBXML2_WITH_TESTS=NO

cmake --build libxml2/build --config=Release --target=install
```

```
cmake -S md-valid -B md-valid/build -D CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded -D CMAKE_INSTALL_PREFIX=md-valid/install

cmake --build md-valid/build --config=Release --target=install
```

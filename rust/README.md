# Build libxml2

```
cmake -S libxml2 -B libxml2/build -D CMAKE_INSTALL_PREFIX=fmi-schema/vendor/x86_64-windows -D CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded -D LIBXML2_WITH_ICONV=OFF -D BUILD_SHARED_LIBS=OFF -D LIBXML2_WITH_PROGRAMS=OFF -D LIBXML2_WITH_TESTS=OFF

cmake --build libxml2/build --config=Release --target=install
```

```
cmake -S md-valid -B md-valid/build -D CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded -D CMAKE_INSTALL_PREFIX=md-valid/install

cmake --build md-valid/build --config=Release --target=install
```

# Chimera Compiler Core - CMake Configuration
#
# This file is part of the Chimera compiler-core build system.

# Chimera version - must match other layers
set(CHIMERA_VERSION_MAJOR 0)
set(CHIMERA_VERSION_MINOR 1)
set(CHIMERA_VERSION_PATCH 0)
set(CHIMERA_VERSION "${CHIMERA_VERSION_MAJOR}.${CHIMERA_VERSION_MINOR}.${CHIMERA_VERSION_PATCH}")

# Feature flags
option(CHIMERA_BUILD_TESTS "Build compiler tests" ON)
option(CHIMERA_BUILD_TOOLS "Build compiler tools" ON)
option(CHIMERA_USE_SHARED_LLVM "Link against shared LLVM libraries" OFF)

# Compiler Core library
add_library(chimera_core INTERFACE)
target_include_directories(chimera_core INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)
target_compile_features(chimera_core INTERFACE cxx_std_17)

# Export version
message(STATUS "Chimera Compiler Core ${CHIMERA_VERSION}")
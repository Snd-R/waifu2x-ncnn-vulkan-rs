cmake_policy(SET CMP0091 NEW)
set(CMAKE_POLICY_DEFAULT_CMP0091 NEW)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")

project(waifu2x-ncnn-vulkan-wrapper)

cmake_minimum_required(VERSION 3.9)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE release CACHE STRING "Choose the type of build" FORCE)
endif ()

find_package(Threads)
find_package(OpenMP)
find_package(Vulkan REQUIRED)
find_package(ncnn REQUIRED)

macro(waifu2x_add_shader SHADER_SRC)
    get_filename_component(SHADER_SRC_NAME_WE ${SHADER_SRC} NAME_WE)
    set(SHADER_COMP_HEADER ${CMAKE_CURRENT_BINARY_DIR}/waifu2x-ncnn-vulkan/src/${SHADER_SRC_NAME_WE}.comp.hex.h)

    add_custom_command(
            OUTPUT ${SHADER_COMP_HEADER}
            COMMAND ${CMAKE_COMMAND} -DSHADER_SRC=${CMAKE_CURRENT_SOURCE_DIR}/waifu2x-ncnn-vulkan/src/${SHADER_SRC} -DSHADER_COMP_HEADER=${SHADER_COMP_HEADER} -P "${CMAKE_CURRENT_SOURCE_DIR}/waifu2x-ncnn-vulkan/src/generate_shader_comp_header.cmake"
            DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/waifu2x-ncnn-vulkan/src/${SHADER_SRC}
            COMMENT "Preprocessing shader source ${SHADER_SRC_NAME_WE}.comp"
            VERBATIM
    )
    set_source_files_properties(${SHADER_COMP_HEADER} PROPERTIES GENERATED TRUE)

    list(APPEND SHADER_SPV_HEX_FILES ${SHADER_COMP_HEADER})
endmacro()

include_directories(${CMAKE_CURRENT_BINARY_DIR}/waifu2x-ncnn-vulkan/src)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/waifu2x-ncnn-vulkan/src)
include_directories(${CMAKE_CURRENT_BINARY_DIR})

if (OPENMP_FOUND)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${OpenMP_C_FLAGS}")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${OpenMP_CXX_FLAGS}")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} ${OpenMP_EXE_LINKER_FLAGS}")
endif ()

# enable global link time optimization
cmake_policy(SET CMP0069 NEW)
set(CMAKE_POLICY_DEFAULT_CMP0069 NEW)
include(CheckIPOSupported)
check_ipo_supported(RESULT ipo_supported OUTPUT ipo_supported_output)
if (ipo_supported)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
else ()
    message(WARNING "IPO is not supported: ${ipo_supported_output}")
endif ()

set(GLSLANG_TARGET_DIR "GLSLANG-NOTFOUND" CACHE PATH "Absolute path to glslangTargets.cmake directory")
if (NOT GLSLANG_TARGET_DIR AND NOT DEFINED ENV{GLSLANG_TARGET_DIR})
    message(FATAL_ERROR "GLSLANG_TARGET_DIR must be defined!")
else ()
    message(STATUS "Using glslang install located at ${GLSLANG_TARGET_DIR}")

    find_package(Threads)

    include("${GLSLANG_TARGET_DIR}/OSDependentTargets.cmake")
    include("${GLSLANG_TARGET_DIR}/OGLCompilerTargets.cmake")
    if (EXISTS "${GLSLANG_TARGET_DIR}/HLSLTargets.cmake")
        # hlsl support can be optional
        include("${GLSLANG_TARGET_DIR}/HLSLTargets.cmake")
    endif ()
    include("${GLSLANG_TARGET_DIR}/glslangTargets.cmake")
    include("${GLSLANG_TARGET_DIR}/SPIRVTargets.cmake")

    if (NOT TARGET glslang OR NOT TARGET SPIRV)
        message(FATAL_ERROR "glslang or SPIRV target not found!")
    endif ()
endif ()

waifu2x_add_shader(waifu2x_preproc.comp)
waifu2x_add_shader(waifu2x_postproc.comp)
waifu2x_add_shader(waifu2x_preproc_tta.comp)
waifu2x_add_shader(waifu2x_postproc_tta.comp)

add_custom_target(generate-spirv DEPENDS ${SHADER_SPV_HEX_FILES})

add_library(waifu2x-ncnn-vulkan-wrapper STATIC waifu2x_wrapped.cpp waifu2x-ncnn-vulkan/src/waifu2x.cpp)

add_dependencies(waifu2x-ncnn-vulkan-wrapper generate-spirv)

set(WAIFU2X_LINK_LIBRARIES ncnn webp ${Vulkan_LIBRARY})

target_link_libraries(waifu2x-ncnn-vulkan-wrapper ${WAIFU2X_LINK_LIBRARIES})

install(TARGETS waifu2x-ncnn-vulkan-wrapper ARCHIVE DESTINATION lib)
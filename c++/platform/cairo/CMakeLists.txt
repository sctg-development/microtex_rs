message(STATUS "Cross-building using GTK")

find_package(Fontconfig REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(Cairo REQUIRED IMPORTED_TARGET cairo)
pkg_check_modules(Pango REQUIRED IMPORTED_TARGET pango)
pkg_check_modules(Pangocairo REQUIRED IMPORTED_TARGET pangocairo)

if (DEFINED _BUILD_STATIC AND _BUILD_STATIC)
  # When building a static microtex library we integrate the Cairo backend
  # directly into the `microtex` target so that a single `libmicrotex.a`
  # contains the functionality needed to render SVG with Cairo.
  target_sources(microtex PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/graphic_cairo.cpp)
  target_include_directories(microtex PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}
                                              ${Cairo_INCLUDE_DIRS}
                                              ${Fontconfig_INCLUDE_DIRS}
                                              ${Pango_INCLUDE_DIRS}
                                              ${Pangocairo_INCLUDE_DIRS})
  target_link_libraries(microtex PRIVATE ${Cairo_LIBRARIES} ${Pango_LIBRARIES} ${Pangocairo_LIBRARIES} ${Fontconfig_LIBRARIES})
  target_compile_definitions(microtex PRIVATE -DHAVE_CAIRO)
else ()
  if (DEFINED _BUILD_STATIC AND _BUILD_STATIC)
    add_library(microtex-cairo STATIC graphic_cairo.cpp)
  else ()
    add_library(microtex-cairo SHARED graphic_cairo.cpp)
  endif ()

  set_target_properties(
      microtex-cairo PROPERTIES
      VERSION ${PROJECT_VERSION}
      SOVERSION ${MICROTEX_API_VERSION}
      CXX_VISIBILITY_PRESET hidden
  )

  # we need to export the library on Windows
  target_compile_definitions(microtex-cairo PRIVATE -DMICROTEX_LIBRARY)
  target_link_libraries(
      microtex-cairo PRIVATE
      microtex
      ${Fontconfig_LIBRARIES}
      ${Cairo_LIBRARIES}
      ${Pango_LIBRARIES}
      ${Pangocairo_LIBRARIES}
      #Fontconfig::Fontconfig
      #PkgConfig::Cairo
      #PkgConfig::Pango
      #PkgConfig::Pangocairo
  )
  include_directories(
    ${Cairo_INCLUDE_DIRS}
    ${Fontconfig_INCLUDE_DIRS}
    ${Pango_INCLUDE_DIRS}
    ${Pangocairo_INCLUDE_DIRS}
  )
endif ()

#microtex_install_target(microtex-cairo)
#microtex_install_headers(microtex-cairo HEADERS graphic_cairo.h)

#include <cairo-svg.h>
#include <cairo.h>

#include <cstdio>
#include <vector>

// build with `g++ ./test_svg_vector.cpp -o ./test_svg_vector $(pkg-config --cflags --libs cairo)`

static cairo_status_t writer(void* closure, const unsigned char* data, unsigned int length) {
  auto v = reinterpret_cast<std::vector<unsigned char>*>(closure);
  v->insert(v->end(), data, data + length);
  return CAIRO_STATUS_SUCCESS;
}

int main() {
  std::vector<unsigned char> vec;
  cairo_surface_t* surface = cairo_svg_surface_create_for_stream(writer, &vec, 200.0, 80.0);
  if (!surface) {
    fprintf(stderr, "surface create failed\n");
    return 1;
  }
  cairo_t* cr = cairo_create(surface);
  if (!cr) {
    fprintf(stderr, "cairo_create failed\n");
    cairo_surface_destroy(surface);
    return 1;
  }

  cairo_select_font_face(cr, "Serif", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);
  cairo_set_font_size(cr, 20.0);
  cairo_move_to(cr, 10, 40);
  cairo_show_text(cr, "Vector test");

  cairo_surface_flush(surface);
  cairo_surface_finish(surface);

  fprintf(stderr, "vec.size=%zu\n", vec.size());
  if (!vec.empty()) {
    FILE* out = fopen("./out_vec.svg", "wb");
    fwrite(vec.data(), 1, vec.size(), out);
    fclose(out);
  }

  cairo_destroy(cr);
  cairo_surface_destroy(surface);
  return 0;
}
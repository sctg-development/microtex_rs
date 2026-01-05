#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <string>
#include <vector>

// microtex was built with
// cd c++ && mkdir -p build && cd build && cmake -DCAIRO=ON -DBUILD_STATIC=ON -DHAVE_CWRAPPER=ON ..
// && make -j${nproc} build with g++ -DHAVE_CWRAPPER ./test_math_svg.cpp ../build/lib/libmicrotex.a
// -o ./test_math_svg -I../lib -I../build/lib $(pkg-config --cflags --libs cairo pangocairo pango
// fontconfig freetype2 harfbuzz)
#include "wrapper/cwrapper.h"

int main() {
  // Read bundled CLM (math font data)
  const char* clm_path = "../res/xits/XITSMath-Regular.clm2";
  std::ifstream f(clm_path, std::ios::binary);
  if (!f) {
    fprintf(stderr, "failed to open clm file: %s\n", clm_path);
    return 1;
  }
  std::vector<unsigned char> clm_data(
    (std::istreambuf_iterator<char>(f)),
    std::istreambuf_iterator<char>()
  );
  if (clm_data.empty()) {
    fprintf(stderr, "empty clm data\n");
    return 1;
  }

  // Initialize MicroTeX
  FontMetaPtr meta = microtex_init(clm_data.size(), clm_data.data());
  if (!meta) {
    fprintf(stderr, "microtex_init failed\n");
    return 2;
  }

  // Optionally set a default main font family to help Cairo/Pango fallback
  microtex_setDefaultMainFont("Serif");

  // Prefer path-based glyph rendering to ensure visual output even when system
  // fonts are not available to the Cairo/Pango stack.
  microtex_setRenderGlyphUsePath(true);
  const char* src = R"LATEX(
  \[
  \iiint\limits_{V} \nabla \cdot \vec{F} \, dV
  = \iint\limits_{S} \vec{F} \cdot \vec{n} \, dS
  \]
)LATEX";

  // Parse and render
  RenderPtr r = microtex_parseRender(src, 720, 20.0f, 20.0f / 3.0f, 0xff000000u, false, false, 0);
  if (!r) {
    fprintf(stderr, "microtex_parseRender failed\n");
    microtex_releaseFontMeta(meta);
    microtex_release();
    return 3;
  }

  unsigned long out_len = 0;
  unsigned char* out_buf = microtex_render_to_svg(r, &out_len);
  if (!out_buf || out_len == 0) {
    fprintf(stderr, "microtex_render_to_svg returned empty buffer\n");
    microtex_deleteRender(r);
    microtex_releaseFontMeta(meta);
    microtex_release();
    return 4;
  }

  FILE* out = fopen("./out_math.svg", "wb");
  if (!out) {
    perror("fopen out_math.svg");
  } else {
    fwrite(out_buf, 1, out_len, out);
    fclose(out);
    fprintf(stderr, "Wrote ./out_math.svg (%lu bytes)\n", out_len);
  }

  microtex_free_buffer(out_buf);
  microtex_deleteRender(r);
  microtex_releaseFontMeta(meta);
  microtex_release();

  return 0;
}

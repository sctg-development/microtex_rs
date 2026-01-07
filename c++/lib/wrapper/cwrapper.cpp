#ifdef HAVE_CWRAPPER

#include "wrapper/cwrapper.h"

#include "microtex.h"
#include "utils/log.h"
#include "wrapper/graphic_wrapper.h"

#include <mutex>
#include <unordered_map>

#ifdef HAVE_CAIRO
#include "../platform/cairo/graphic_cairo.h"
#include <cairo.h>
#include <cairo-svg.h>
#include <cstring>
#endif

// Buffer reference counting to avoid double-free across FFI boundary
static std::mutex __buf_ref_mutex;
static std::unordered_map<unsigned char *, int> __buf_refcounts;

using namespace microtex;

#ifdef __cplusplus
extern "C"
{
#endif

  MICROTEX_CAPI const char *microtex_version()
  {
    // No need to copy, [MicroTeX::version] returns a static string
    return MicroTeX::version().c_str();
  }

  MICROTEX_CAPI void microtex_registerCallbacks(
      CBCreateTextLayout createTextLayout,
      CBGetTextLayoutBounds getTextLayoutBounds,
      CBReleaseTextLayout releaseTextLayout,
      CBIsPathExists isPathExists)
  {
    microtex_createTextLayout = createTextLayout;
    microtex_getTextLayoutBounds = getTextLayoutBounds;
    microtex_releaseTextLayout = releaseTextLayout;
    microtex_isPathExists = isPathExists;
  }

  MICROTEX_CAPI void
  microtex_setTextLayoutBounds(TextLayoutBounds *b, float width, float height, float ascent)
  {
    b->width = width;
    b->height = height;
    b->ascent = ascent;
  }

  MICROTEX_CAPI bool microtex_isBold(FontDesc *desc)
  {
    return desc->isBold;
  }

  MICROTEX_CAPI bool microtex_isItalic(FontDesc *desc)
  {
    return desc->isItalic;
  }

  MICROTEX_CAPI bool microtex_isSansSerif(FontDesc *desc)
  {
    return desc->isSansSerif;
  }

  MICROTEX_CAPI bool microtex_isMonospace(FontDesc *desc)
  {
    return desc->isMonospace;
  }

  MICROTEX_CAPI float microtex_fontSize(FontDesc *desc)
  {
    return desc->fontSize;
  }

  MICROTEX_CAPI FontMetaPtr microtex_init(unsigned long len, const unsigned char *data)
  {
    auto factory = std::make_unique<PlatformFactory_wrapper>();
    PlatformFactory::registerFactory("__wrapper__", std::move(factory));
    PlatformFactory::activate("__wrapper__");
    FontSrcData src{len, data};
    auto meta = MicroTeX::init(src);
    return new FontMeta(meta);
  }

  MICROTEX_CAPI void microtex_release()
  {
    MicroTeX::release();
  }

  MICROTEX_CAPI bool microtex_isInited()
  {
    return MicroTeX::isInited();
  }

  MICROTEX_CAPI FontMetaPtr microtex_addFont(unsigned long len, const unsigned char *data)
  {
    FontSrcData src{len, data};
    auto meta = MicroTeX::addFont(src);
    // create a new FontMeta from heap
    // [microtex_releaseFontMeta] must be called after this object has no usages.
    return new FontMeta(meta);
  }

  MICROTEX_CAPI const char *microtex_getFontFamily(FontMetaPtr ptr)
  {
    auto *meta = (FontMeta *)ptr;
    // no need to copy
    return meta->family.c_str();
  }

  MICROTEX_CAPI const char *microtex_getFontName(FontMetaPtr ptr)
  {
    auto *meta = (FontMeta *)ptr;
    // no need to copy
    return meta->name.c_str();
  }

  MICROTEX_CAPI bool microtex_isMathFont(FontMetaPtr ptr)
  {
    auto *meta = (FontMeta *)ptr;
    return meta->isMathFont;
  }

  MICROTEX_CAPI void microtex_releaseFontMeta(FontMetaPtr ptr)
  {
    auto *meta = (FontMeta *)ptr;
    delete meta;
  }

  MICROTEX_CAPI void microtex_setDefaultMathFont(const char *name)
  {
    MicroTeX::setDefaultMathFont(name);
  }

  MICROTEX_CAPI void microtex_setDefaultMainFont(const char *name)
  {
    MicroTeX::setDefaultMainFont(name);
  }

  MICROTEX_CAPI bool microtex_hasGlyphPathRender()
  {
    return MicroTeX::hasGlyphPathRender();
  }

  MICROTEX_CAPI void microtex_setRenderGlyphUsePath(bool use)
  {
    return MicroTeX::setRenderGlyphUsePath(use);
  }

  MICROTEX_CAPI bool microtex_isRenderGlyphUsePath()
  {
    return MicroTeX::isRenderGlyphUsePath();
  }

  MICROTEX_CAPI RenderPtr microtex_parseRender(
      const char *tex,
      int width,
      float textSize,
      float lineSpace,
      unsigned int color,
      bool fillWidth,
      bool enableOverrideTeXStyle,
      unsigned int texStyle)
  {
#ifdef HAVE_LOG
    logv("parse: %s\n", tex);
#endif
    auto r = MicroTeX::parse(
        tex,
        width,
        textSize,
        lineSpace,
        color,
        fillWidth,
        {enableOverrideTeXStyle, static_cast<TexStyle>(texStyle)});
    return reinterpret_cast<RenderPtr>(r);
  }

  MICROTEX_CAPI void microtex_deleteRender(RenderPtr render)
  {
    auto r = reinterpret_cast<Render *>(render);
    delete r;
  }

  MICROTEX_CAPI DrawingData microtex_getDrawingData(RenderPtr render, int x, int y)
  {
    auto r = reinterpret_cast<Render *>(render);
    Graphics2D_wrapper g2;
    r->draw(g2, x, y);
    return g2.getDrawingData();
  }

  MICROTEX_CAPI void microtex_freeDrawingData(DrawingData data)
  {
    free(data);
  }

  MICROTEX_CAPI bool microtex_isLittleEndian()
  {
    int n = 1;
    return *((char *)&n) == 1;
  }

  MICROTEX_CAPI int microtex_getRenderWidth(RenderPtr render)
  {
    auto r = reinterpret_cast<Render *>(render);
    return r->getWidth();
  }

  MICROTEX_CAPI int microtex_getRenderHeight(RenderPtr render)
  {
    auto r = reinterpret_cast<Render *>(render);
    return r->getHeight();
  }

  MICROTEX_CAPI int microtex_getRenderDepth(RenderPtr render)
  {
    auto r = reinterpret_cast<Render *>(render);
    return r->getDepth();
  }

  MICROTEX_CAPI bool microtex_isRenderSplit(RenderPtr render)
  {
    auto r = reinterpret_cast<Render *>(render);
    return r->isSplit();
  }

  MICROTEX_CAPI void microtex_setRenderTextSize(RenderPtr render, float size)
  {
    auto r = reinterpret_cast<Render *>(render);
    r->setTextSize(size);
  }

  MICROTEX_CAPI void microtex_setRenderForeground(RenderPtr render, color c)
  {
    auto r = reinterpret_cast<Render *>(render);
    r->setForeground(c);
  }

#ifdef HAVE_CAIRO

  // C-compatible writer used by cairo_svg_surface_create_for_stream. We use a
  // plain function (not a lambda) to avoid ABI incompatibilities on some
  // platforms when passing function pointers to C APIs.
  static cairo_status_t svg_writer_func(void *closure, const unsigned char *data, unsigned int length)
  {
    auto v = reinterpret_cast<std::vector<unsigned char> *>(closure);
    v->insert(v->end(), data, data + length);
    return CAIRO_STATUS_SUCCESS;
  }

  MICROTEX_CAPI unsigned char *microtex_render_to_svg(RenderPtr render, unsigned long *out_len)
  {
    auto r = reinterpret_cast<Render *>(render);
    std::vector<unsigned char> vec;

    cairo_surface_t *surface = cairo_svg_surface_create_for_stream(
        svg_writer_func,
        &vec,
        (double)r->getWidth(),
        (double)r->getHeight());
    if (!surface)
    {
      fprintf(stderr, "microtex_render_to_svg: failed to create cairo surface\n");
    }
    cairo_t *cr = cairo_create(surface);
    if (!cr)
    {
      fprintf(stderr, "microtex_render_to_svg: failed to create cairo context\n");
    }

    microtex::Graphics2D_cairo g2(cr);
    r->draw(g2, 0, 0);

    // ensure the SVG stream is flushed/written
    cairo_surface_flush(surface);
    cairo_surface_finish(surface);

    cairo_destroy(cr);
    cairo_surface_destroy(surface);

    if (out_len)
      *out_len = vec.size();
    if (vec.empty())
    {
      fprintf(stderr, "microtex_render_to_svg: returning NULL (empty buffer)\n");
      return nullptr;
    }
    unsigned char *out = (unsigned char *)malloc(vec.size());
    if (!out)
    {
      fprintf(stderr, "microtex_render_to_svg: malloc failed for %zu bytes\n", vec.size());
      return nullptr;
    }
    memcpy(out, vec.data(), vec.size());
    // register buffer with refcount = 1
    {
      std::lock_guard<std::mutex> lg(__buf_ref_mutex);
      __buf_refcounts[out] = 1;
    }
    return out;
  }

  // Helper function to create a simple JSON string with render metrics
  // This avoids depending on external JSON libraries
  static std::string render_metrics_to_json(Render *r, const std::string &svg_content)
  {
    // Escape SVG content for JSON by replacing quotes and backslashes
    std::string escaped_svg;
    for (char c : svg_content)
    {
      if (c == '"')
        escaped_svg += "\\\"";
      else if (c == '\\')
        escaped_svg += "\\\\";
      else if (c == '\n')
        escaped_svg += "\\n";
      else if (c == '\r')
        escaped_svg += "\\r";
      else
        escaped_svg += c;
    }

    // Build JSON object with SVG and metrics
    char json_buffer[4096];
    int written = snprintf(
        json_buffer,
        sizeof(json_buffer),
        "{"
        "\"svg\":\"%s\","
        "\"metrics\":{"
        "\"width\":%d,"
        "\"height\":%d,"
        "\"depth\":%d,"
        "\"ascent\":%d"
        "}"
        "}",
        escaped_svg.c_str(),
        r->getWidth(),
        r->getHeight() + r->getDepth(),
        r->getDepth(),
        r->getHeight());

    if (written < 0 || written >= (int)sizeof(json_buffer))
    {
      fprintf(stderr, "render_metrics_to_json: JSON buffer overflow\n");
      return "{}";
    }

    return std::string(json_buffer);
  }

  MICROTEX_CAPI unsigned char *microtex_render_to_svg_with_metrics(RenderPtr render, unsigned long *out_len)
  {
    auto r = reinterpret_cast<Render *>(render);
    if (!r)
    {
      fprintf(stderr, "microtex_render_to_svg_with_metrics: invalid render pointer\n");
      if (out_len)
        *out_len = 0;
      return nullptr;
    }

    std::vector<unsigned char> svg_vec;

    // Generate SVG content
    cairo_surface_t *surface = cairo_svg_surface_create_for_stream(
        svg_writer_func,
        &svg_vec,
        (double)r->getWidth(),
        (double)r->getHeight());
    if (!surface)
    {
      fprintf(stderr, "microtex_render_to_svg_with_metrics: failed to create cairo surface\n");
      if (out_len)
        *out_len = 0;
      return nullptr;
    }

    cairo_t *cr = cairo_create(surface);
    if (!cr)
    {
      fprintf(stderr, "microtex_render_to_svg_with_metrics: failed to create cairo context\n");
      cairo_surface_destroy(surface);
      if (out_len)
        *out_len = 0;
      return nullptr;
    }

    microtex::Graphics2D_cairo g2(cr);
    r->draw(g2, 0, 0);

    // Ensure the SVG stream is flushed/written
    cairo_surface_flush(surface);
    cairo_surface_finish(surface);

    cairo_destroy(cr);
    cairo_surface_destroy(surface);

    // Convert SVG vector to string
    std::string svg_str(svg_vec.begin(), svg_vec.end());

    // Create JSON with SVG and metrics
    std::string json_str = render_metrics_to_json(r, svg_str);

    if (json_str.empty() || json_str == "{}")
    {
      fprintf(stderr, "microtex_render_to_svg_with_metrics: failed to create JSON\n");
      if (out_len)
        *out_len = 0;
      return nullptr;
    }

    // Allocate and copy result
    unsigned char *out = (unsigned char *)malloc(json_str.size());
    if (!out)
    {
      fprintf(stderr, "microtex_render_to_svg_with_metrics: malloc failed for %zu bytes\n", json_str.size());
      if (out_len)
        *out_len = 0;
      return nullptr;
    }

    memcpy(out, json_str.c_str(), json_str.size());

    // Register buffer with refcount = 1
    {
      std::lock_guard<std::mutex> lg(__buf_ref_mutex);
      __buf_refcounts[out] = 1;
    }

    if (out_len)
      *out_len = json_str.size();

    return out;
  }

#else

// Stub implementation when HAVE_CAIRO is not defined
MICROTEX_CAPI unsigned char *microtex_render_to_svg(RenderPtr render, unsigned long *out_len)
{
  fprintf(stderr, "microtex_render_to_svg: Cairo support not compiled\n");
  if (out_len)
    *out_len = 0;
  return nullptr;
}

MICROTEX_CAPI unsigned char *microtex_render_to_svg_with_metrics(RenderPtr render, unsigned long *out_len)
{
  fprintf(stderr, "microtex_render_to_svg_with_metrics: Cairo support not compiled\n");
  if (out_len)
    *out_len = 0;
  return nullptr;
}

#endif

  MICROTEX_CAPI void microtex_free_buffer(unsigned char *buf)
  {
    if (!buf)
      return;
    std::lock_guard<std::mutex> lg(__buf_ref_mutex);
    auto it = __buf_refcounts.find(buf);
    if (it == __buf_refcounts.end())
    {
      // unknown buffer, free directly but log it
      fprintf(stderr, "microtex_free_buffer: freeing unknown buffer %p\n", (void *)buf);
      free(buf);
      return;
    }
    it->second -= 1;
    if (it->second <= 0)
    {
      free(buf);
      __buf_refcounts.erase(it);
    }
    else
    {
      // not the last reference; do not free
    }
  }

  MICROTEX_CAPI void microtex_retain_buffer(unsigned char *buf)
  {
    if (!buf)
      return;
    std::lock_guard<std::mutex> lg(__buf_ref_mutex);
    auto it = __buf_refcounts.find(buf);
    if (it == __buf_refcounts.end())
    {
      // unknown buffer; register with refcount 1
      __buf_refcounts[buf] = 1;
    }
    else
    {
      it->second += 1;
    }
  }

#ifdef __cplusplus
}
#endif
#endif // HAVE_CWRAPPER

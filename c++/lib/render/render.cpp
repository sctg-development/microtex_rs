#include "render.h"

#include <functional>
#include <vector>

#include "atom/atom.h"
#include "box/box_single.h"
#include "box/box_group.h"
#include "core/debug_config.h"
#include "core/split.h"
#include "env/env.h"

using namespace std;
using namespace microtex;

namespace microtex {

using BoxFilter = std::function<bool(const sptr<Box>&)>;

struct RenderData {
  sptr<Box> root;
  float textSize;
  float fixedScale;
  color fg;
  bool isSplit;
};

static sptr<BoxGroup> wrap(const sptr<Box>& box) {
  sptr<BoxGroup> parent;
  if (auto group = dynamic_pointer_cast<BoxGroup>(box); group != nullptr) {
    parent = group;
  } else {
    parent = sptrOf<HBox>(box);
  }
  return parent;
}

static void
buildDebug(const sptr<BoxGroup>& parent, const sptr<Box>& box, const BoxFilter& filter) {
  if (parent != nullptr) {
    if (box->isSpace()) {
      parent->addOnly(box);
    } else if (filter(box)) {
      parent->addOnly(sptrOf<DebugBox>(box));
    } else {
      // placeholder to consume the space of the current box
      parent->addOnly(sptrOf<StrutBox>(box));
    }
  }
  if (auto group = dynamic_pointer_cast<BoxGroup>(box); group != nullptr) {
    const auto kern =
      sptrOf<StrutBox>(-group->_width, -group->_height, -group->_depth, -group->_shift);
    // snapshot of current children
    const auto children = group->descendants();
    group->addOnly(kern);
    for (const auto& child : children) {
      buildDebug(group, child, filter);
    }
  } else if (auto decor = dynamic_pointer_cast<DecorBox>(box); decor != nullptr) {
    const auto g = wrap(decor->_base);
    decor->_base = g;
    buildDebug(nullptr, g, filter);
  }
}

}  // namespace microtex

Render::Render(const sptr<Box>& box, float textSize, bool isSplit) {
  _data = new RenderData{box, textSize, textSize / Env::fixedTextSize(), black, isSplit};
  const auto& debugConfig = DebugConfig::INSTANCE;
  if (debugConfig.enable) {
    const auto group = microtex::wrap(box);
    _data->root = group;
    BoxFilter filter = [&](const sptr<Box>& b) {
      return (
        debugConfig.showOnlyChar ? dynamic_cast<CharBox*>(b.get()) != nullptr : !b->isSpace()
      );
    };
    microtex::buildDebug(nullptr, group, filter);
  }
}

Render::~Render() {
  delete _data;
}

float Render::getTextSize() const {
  return _data->textSize;
}

int Render::getHeight() const {
  auto box = _data->root;
  return (int)(box->vlen() * _data->fixedScale);
}

int Render::getDepth() const {
  return (int)(_data->root->_depth * _data->fixedScale);
}

int Render::getWidth() const {
  return (int)(_data->root->_width * _data->fixedScale);
}

float Render::getBaseline() const {
  auto box = _data->root;
  return box->_height / box->vlen();
}

bool Render::isSplit() const {
  return _data->isSplit;
}

void Render::setTextSize(float textSize) {
  _data->textSize = textSize;
  _data->fixedScale = textSize / Env::fixedTextSize();
}

void Render::setForeground(color fg) {
  _data->fg = fg;
}

void Render::draw(Graphics2D& g2, int x, int y) {
  color old = g2.getColor();
  auto fixedScale = _data->fixedScale;
  auto box = _data->root;

  g2.setColor(isTransparent(_data->fg) ? black : _data->fg);
  g2.translate(x, y);
  g2.scale(fixedScale, fixedScale);

  // draw formula box
  box->draw(g2, 0, box->_height);

  // restore
  g2.scale(1.f / fixedScale, 1.f / fixedScale);
  g2.translate(-x, -y);
  g2.setColor(old);
}
void Render::getKeyCharMetrics(std::vector<int>& heights, std::vector<int>& depths) {
  heights.clear();
  depths.clear();

  if (!_data || !_data->root) {
    return;
  }

  auto root = _data->root;

  // Recursive helper function to extract CharBox heights
  std::function<void(const std::shared_ptr<Box>&)> extractCharBoxes;
  extractCharBoxes = [&](const std::shared_ptr<Box>& box) {
    if (!box) return;
    
    // Try to cast to CharBox first
    if (auto charbox = dynamic_pointer_cast<CharBox>(box); charbox != nullptr) {
      if (charbox->_height > 0) {  // Only add positive heights
        heights.push_back(static_cast<int>(charbox->_height));
        depths.push_back(static_cast<int>(charbox->_depth));
      }
      return;  // Don't recurse into CharBox
    }
    
    // Try to cast to BoxGroup and recurse on children
    if (auto group = dynamic_pointer_cast<BoxGroup>(box); group != nullptr) {
      for (const auto& child : group->_children) {
        extractCharBoxes(child);  // Recurse
      }
      return;
    }
  };

  extractCharBoxes(root);
}

float Render::getBoxTreeHeight() const {
  if (!_data || !_data->root) {
    return 0.0f;
  }
  
  auto root = _data->root;
  
  // Get the height from the root box, which is the total height in MicroTeX units
  // This includes ascent + depth
  return root->_height;
}
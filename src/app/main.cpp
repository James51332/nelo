#include "renderer/context.h"

#include <memory>

int main()
{
  std::unique_ptr<nelo::context> context = std::make_unique<nelo::context>(false);
  while (context->active())
    context->update();
}

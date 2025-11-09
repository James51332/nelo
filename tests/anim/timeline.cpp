#include <nelo/anim/timeline.h>
using namespace nelo;

#include <cassert>
#include <glm/gtc/epsilon.hpp>

int anim_timeline(int argc, char** const argv)
{
  // Sample with a double for now.
  nelo::timeline x = 5;
  nelo::timeline y = [](double t) { return t; };

  // We use epsilon comparsion due to floating point precision errors.
  double e = 0.0001;
  assert(glm::epsilonEqual(x.sample(-1.0), 5.0, e));
  assert(glm::epsilonEqual(y.sample(5.0), 5.0, e));

  // Here we can get some values to do the testing.
  x.add_timeline(0.5, y);
  assert(glm::epsilonEqual(x.sample(1.5), 6.0, e));

  // We should not be able to add x onto y anymore.
  try
  {
    y.multiply_timeline(4.0, x);

    // This is a bad pipeline
    // assert(false);
  }
  catch (std::runtime_error& e)
  {
    // No need to do anything.
  }

  return 0;
}

#include "traits.h"

namespace nelo
{

// We'll go ahead and define some timeline traits for common objects.
template <>
struct timeline_traits<double>
{
  double lerp(double a, double b, double t) { return a + (b - a) * t; }
  double add(double a, double b) { return a + b; }
  double multiply(double a, double b) { return a * b; }
};

} // namespace nelo

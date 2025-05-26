#include <iostream>

#include "timeline.h"

int main()
{
  nelo::timeline<double> t(4.0);
  auto t1 = std::make_shared<nelo::timeline<double>>([](double t) -> double { return t; });
  t.add_timeline(0.0, t1);
  std::cout << "Hello, world!" << std::endl;
}

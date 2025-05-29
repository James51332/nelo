#include <glad/glad.h>
#include <iostream>
#include <nelo/renderer/context.h>
#include <nelo/scene/timeline.h>

int main()
{
  // Create our nelo render context in a windowed mode.
  std::unique_ptr<nelo::context> context = std::make_unique<nelo::context>(false);

  // Let's do some testing with the timeline API for the time being.
  nelo::timeline x = 5;
  nelo::timeline y = [](double t) { return t; };

  // Here we can get some values to do the testing.
  std::cout << "Sampled timeline x at t=1.0 and got " << x.sample(1.0) << std::endl;
  x.add_timeline(0.0, y);
  std::cout << "Sampled timeline x + y at t=4.0 and got " << x.sample(4.0) << std::endl;
  std::cout << "Sampled timeline y at t=4.0 and got " << y.sample(4.0) << std::endl;

  // Update the window until we are closed, rendering via raw OpenGL for now.
  while (context->active())
  {
    // Poll events.
    context->update();

    // Render and present to the window.
    glClearColor(0.2f, 0.2f, 0.2f, 1.0f);
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    context->present();
  }
}

-- This is an optional setup when using internal engine. Needed when using nelo in any other lua tool.
require("nelo").setup_globals()

-- Create our scene. The name is video file name. 
local hello_nelo = scene('Hello Nelo')

local object = hello_nelo:create_entity {
  circle = circle {
    fill_color = color(1.0, 1.0, 0.0, 1.0),
    radius = timeline('number', function(t) return 1.5 + 0.5 * math.sin(math.pi * t) end),
  },
  transform = transform {
    position = timeline('vec3', function(t) return vec3(2.0 * math.cos(4.0 * t), 2.0 * math.sin(4.0 * t), 0.0) end)
  }
}

-- Render our scene for 10 seconds.
hello_nelo:play(10.0)

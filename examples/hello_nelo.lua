-- This is an optional setup when using internal engine. Needed when using nelo in any other lua tool.
require("nelo").setup_globals()

-- Create our scene. The name is video file name. 
local hello_nelo = scene.new('Hello Nelo')

-- We'll create a better API in the future. This is all pretty tentative.
local circ = circle.new()
circ.radius = timeline('number', function(t) return 1.5 + 0.5 * math.sin(math.pi * t) end)

local trans = transform.new()
trans.position = timeline('vec3', function(t) return vec3.new(2.0 * math.cos(4.0 * t), 2.0 * math.sin(4.0 * t), 0.0) end)

local obj = hello_nelo:create_entity()
obj.circle = circ
obj.transform = trans

-- Render our scene for 10 seconds.
hello_nelo:play(10.0)

-- Our first scene in nelo using lua. Let's give it a test.
local hello_nelo = scene.new('Hello Nelo')

local circ = circle.new()
circ.radius = timeline('number', function(t) return 1.5 + 0.5 * math.sin(math.pi * t) end)

local trans = transform.new()
trans.position = timeline('vec3', function(t) return vec3.new(2.0 * math.cos(4.0 * t), 2.0 * math.sin(4.0 * t), 0.0) end)

local obj = hello_nelo:create_entity()
obj.circle = circ
obj.transform = trans

-- Render our scene for 10 seconds.
hello_nelo:play(10.0)

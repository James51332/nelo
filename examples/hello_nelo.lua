-- Our first scene in nelo using lua. Let's give it a test.
local hello_nelo = scene.new('Hello Nelo')

local path = timeline('vec3', function(t) return vec3.new(5 * math.sin(5.0 * t), 0, 0) end)
hello_nelo:set_path(path)

-- Render our scene for 10 seconds.
hello_nelo:play(10.0)

-- Show how we can load nelo as a module. This also works in the nelo executable.
local nelo = require 'nelo'
local scene = nelo.scene('module')
scene:play(2)

-- We can also setup globals to use the library as if it were the actual tool.
nelo.setup_globals()
local tmp = circle()


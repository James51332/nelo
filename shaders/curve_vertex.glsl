#version 410 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec4 color;
layout (location = 2) in double stroke;
layout (location = 3) in double alpha;

layout (location = 0) out vec4 tmp_col;

uniform vec2 viewport_size;
uniform float scene_height;

void main()
{
  tmp_col = color;
  
  // Scale the width by the aspect ratio.
  vec3 screen_pos = pos;
  screen_pos.x *= viewport_size.y / viewport_size.x;
  gl_Position = vec4(screen_pos.xy / scene_height, screen_pos.z, 1.0);
})

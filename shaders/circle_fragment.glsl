#version 410 core

layout (location = 0) in vec4 tmp_col;
layout (location = 1) in vec2 tmp_uv;

out vec4 frag_color;

void main()
{
  // To determine if we are in the circle, we can use the uv coordinates. We'll map from -1 to 1.
  // float dist = length(tmp_uv);
  // float delta = fwidth(dist);
  // float alpha = smoothstep(1.0 + delta, 1.0 - delta, dist);
  frag_color = vec4(1.0); //vec4(tmp_col.xyz, tmp_col.w * alpha);
}

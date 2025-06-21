#version 410 core

layout (location = 0) in vec4 tmp_col;

out vec4 frag_color;

void main()
{
  frag_color = tmp_col;
}

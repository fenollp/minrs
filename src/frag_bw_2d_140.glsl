#version 140

uniform vec3 window;
uniform sampler2D tex;

in vec2 pos;
out vec4 color;

void main() {
    vec4 c = texture(tex, pos);
    color = vec4(c.r, c.r, c.r, 1);
}

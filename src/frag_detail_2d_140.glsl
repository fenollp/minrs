#version 140

uniform vec3 window;
uniform sampler2D tex;
uniform sampler1D tex_detail;

in vec2 pos;
out vec4 color;

void main() {
    float pos1d = texture(tex, pos).r;
    vec3 c = texture(tex_detail, pos1d).rgb;
    color = vec4(c, 1);
}

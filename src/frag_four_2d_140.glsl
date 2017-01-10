#version 140

uniform vec3 window;
uniform sampler2D tex;

in vec2 pos;
out vec4 color;

vec3 four(in float c) {
    if (c == 0.0) return vec3(0.0, 0.0, 0.0);
    if (c == 1.0) return vec3(1.0, 1.0, 1.0);
    if ((c <=  9.0/255.0 && c <=  13.0/255.0) ||
        (c <= 32.0/255.0 && c <= 126.0/255.0))
        return vec3(55.0/255.0, 126.0/255.0, 184.0/255.0);
    return vec3(228.0/255.0, 26.0/255.0, 28.0/255.0);
}

void main() {
    float c = texture(tex, pos).r;
    color = vec4(four(c), 1);
}

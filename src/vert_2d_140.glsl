#version 140

uniform vec3 window;

in vec2 position;
out vec2 pos;

void main() {
    gl_PointSize = 1;
    gl_Position = vec4(position, 0, 1);
    pos = position;
}

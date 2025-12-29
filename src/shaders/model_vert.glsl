#version 460 core

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 nor;
layout (location = 2) in vec2 uv;

out vec3 vnor;
out vec2 vuv;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

void main() {
    vnor = nor;
    vuv = uv;

    gl_Position = proj * view * model * vec4(pos, 1.0);
}

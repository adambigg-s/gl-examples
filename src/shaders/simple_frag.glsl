#version 460 core

in vec3 vcol;

out vec4 fcol;

uniform mat4 dfdf;

void main() {
    fcol = vec4(vcol, 1.0);
}

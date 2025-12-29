#version 460 core

in vec3 vcol;

out vec4 fcol;

void main() {
    fcol = vec4(vcol, 1.0);
}

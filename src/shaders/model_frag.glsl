#version 460 core

in vec3 vnor;
in vec2 vuv;

out vec4 fcol;

uniform sampler2D tex;

void main() {
    // fcol = texture(tex, vuv);

    fcol = vec4(vnor, 1.0);
}

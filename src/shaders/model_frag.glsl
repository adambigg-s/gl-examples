#version 460 core

in vec4 vnor;
in vec2 vuv;
in vec3 vpos;

out vec4 fcol;

uniform sampler2D tex;

void main() {
    // fcol = texture(tex, vuv);
    fcol = vec4(vpos, 1.0);
}

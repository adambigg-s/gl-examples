#version 460 core

in vec3 vnor;
in vec2 vuv;

out vec4 fcol;

uniform sampler2D tex;
uniform vec3 light;

void main() {
    vec4 texcolor = texture(tex, vuv);
    float brightness = max(dot(normalize(light), vnor), 0.1);

    fcol = texcolor * brightness;
}

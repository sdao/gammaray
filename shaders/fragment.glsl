#version 150 core

uniform sampler2D t_Texture;

in vec2 v_St;
out vec4 Target0;

void main() {
    vec2 clip = step(0.0, v_St) * step(-1.0, -v_St);

    vec4 fill = vec4(1.0, 1.0, 1.0, 1.0);

    vec4 image = texture(t_Texture, v_St);

    Target0 = mix(fill, image, clip.x * clip.y);
}

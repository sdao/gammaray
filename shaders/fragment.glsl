#version 150 core

uniform sampler2D t_Texture;

in vec2 v_St;
out vec4 Target0;

uniform Transform {
    vec2 u_WindowSize;
    vec2 u_ImageSize;
};

void main() {
    vec2 clip = step(0.0, v_St) * step(-1.0, -v_St);

    vec2 square_coords = vec2(v_St.x * u_ImageSize.x, v_St.y * u_ImageSize.y);
    vec2 checker = floor(square_coords / 8.0);
    vec4 fill = mod(checker.x + checker.y, 2.0) == 0.0
            ? vec4(1.0, 1.0, 1.0, 1.0) : vec4(0.9, 0.9, 0.9, 1.0);

    vec4 image = texture(t_Texture, v_St);

    Target0 = mix(fill, image, clip.x * clip.y);
}

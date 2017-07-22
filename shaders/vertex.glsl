#version 150 core

in vec2 a_Pos;
in vec2 a_St;
out vec2 v_St;

uniform Transform {
    vec2 u_WindowSize;
    vec2 u_ImageSize;
};

void main() {
    vec2 diff = u_WindowSize - u_ImageSize;
    vec2 half_diff = diff * 0.5;
    vec2 pixel_offset = (half_diff - floor(half_diff)) / u_ImageSize;

    vec2 image_scale = u_WindowSize / u_ImageSize;
    vec2 image_pos = (a_St - vec2(0.5)) * image_scale + vec2(0.5);

    vec2 image_pos_with_offset = image_pos + pixel_offset;

    gl_Position = vec4(a_Pos, 0.0, 1.0);
    v_St = image_pos_with_offset;
}

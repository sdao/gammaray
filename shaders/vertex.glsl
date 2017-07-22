#version 150 core

in vec2 a_Pos;
in vec2 a_St;
out vec2 v_St;

uniform Transform {
    vec2 u_WindowSize;
    vec2 u_ImageSize;
};

void main() {
    vec2 image_scale = u_WindowSize / u_ImageSize;
    vec2 image_pos = (a_St - vec2(0.5)) * image_scale + vec2(0.5);

    gl_Position = vec4(a_Pos, 0.0, 1.0);
    v_St = image_pos;
}

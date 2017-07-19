#version 150 core

in vec2 a_Pos;
in vec2 a_St;
out vec2 v_St;

uniform Transform {
    vec4 u_WindowFrame; /* left, top, right, bottom */
};

void main() {
    v_St = a_St;

    vec4 pos;
    pos.x = mix(-1.0 + 2.0 * u_WindowFrame.x,
                 1.0 - 2.0 * u_WindowFrame.z,
                 0.5 * (a_Pos.x + 1.0));
    pos.y = mix(-1.0 + 2.0 * u_WindowFrame.w,
                 1.0 - 2.0 * u_WindowFrame.y,
                 0.5 * (a_Pos.y + 1.0));
    pos.z = 0.0;
    pos.w = 1.0;
    gl_Position = pos;
}

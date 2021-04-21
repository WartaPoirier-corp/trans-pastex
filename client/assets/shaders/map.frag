#version 450

layout(location = 0) in vec4 v_Pos;
layout(location = 1) in flat vec4 v_Color;
layout(location = 0) out vec4 o_Target;

void main() {
    vec3 col = v_Color.xyz * (1.0 - v_Pos.y / 2.0);
    o_Target = vec4(col, 1.0);
}

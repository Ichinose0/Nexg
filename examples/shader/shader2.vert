#version 450

#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform Ubo {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec4 inPos;
layout(location = 1) in vec4 inColor;
layout(location = 0) out vec4 fragmentColor;

void main() {
    gl_Position = ubo.proj * ubo.view * ubo.model * inPos;
    fragmentColor = inColor;
}
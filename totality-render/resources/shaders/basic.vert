#version 450

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 norm;
layout (location = 2) in vec2 uv;

struct MeshData {
    mat4 orientation;
    vec3 offset;
};
struct Light {
    vec4 color_and_kind;
    vec4 offset;
};

layout (set = 0, binding = 0, std140) uniform Counts {
    layout(offset = 0) uvec4 count; // instance, light, material, spare
} counts;
layout (set = 0, binding = 1, std140) uniform PerMeshData {
    layout (offset =  0) mat4 offset_orientations[1024];
} per_mesh_data;
layout (set = 0, binding = 2, std140) uniform Lights {
    layout (offset = 0) Light lights[1024];
} lights_data;
layout (set = 0, binding = 3, std140) uniform Materials {
    layout (offset =  0) vec4 materials[1024];
} material_data;
layout (set = 0, binding = 4, std140) uniform Wireframe {
  layout (offset = 0) bool draw_wireframe;
} wireframe;
layout (set = 0, binding = 5, std140) uniform Camera {
  layout (offset =  0) mat4 offori;
} camera;

layout (location = 0) out gl_PerVertex {
    vec4 gl_Position;
    float gl_PointSize;
    float gl_ClipDistance[];
};
layout (location = 1) out vec2 vert_uv;
layout (location = 2) out vec3 vert_norm;
layout (location = 3) out vec3 vert_pos;

void main() {
    mat4 model_offori = per_mesh_data.offset_orientations[gl_InstanceIndex];
    vec4 world_pos = model_offori * vec4(position, 1);

    gl_Position = camera.offori * world_pos;
    vert_uv = uv;
    vert_norm = norm;
    vert_pos = vec3(world_pos);

    // passthrough for debug
    // gl_Position = vec4(position, 1);
    // vert_uv = uv;
    // vert_norm = norm;
    // vert_pos = vec3(world_pos);
}

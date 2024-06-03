#version 450

layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 vert_norm;
layout (location = 3) in vec2 normalized_bc;
layout (location = 4) in vec3 height_adjusted_bc;
layout (location = 5) in vec3 vert_pos;

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

layout(set = 1, binding = 0) uniform texture2D tex;
layout(set = 1, binding = 1) uniform sampler samp;

layout (location = 0) out vec4 color;

void main() {
    // Wireframe drawing. Takes precedence over all shading.
    if (wireframe.draw_wireframe) {
        // If a barymetric coordinate is "small", we're close to the edge. Otherwise, proceed to normal shading.
        float distance_to_closest_edge = min(min(height_adjusted_bc.x, height_adjusted_bc.y), height_adjusted_bc.z);
        if (distance_to_closest_edge < 0.01) {
            color = vec4(0, 1, 0, 1);
            return;
        }
    }

    vec3 diffuse = vec3(uv, 0.0);
    ivec2 texSize = textureSize(sampler2D(tex, samp), 0);
    if (texSize.x != 1 && texSize.y != 1) {
        diffuse = vec3(texture(sampler2D(tex, samp), uv));
    }

    color = vec4(0, 0, 0, 1);
    // Iterate lights.
    int i;
    for (i = 0; i < counts.count[1]; i++) {
        vec3 light_color = vec3(lights_data.lights[i].color_and_kind);
        float kind = lights_data.lights[i].color_and_kind[3];
        if (kind == 1) {
            // Point light
            vec3 to_surface = vert_pos - vec3(lights_data.lights[i].offset);
            float distance = length(to_surface);
            vec3 effective_direction = normalize(to_surface);
            float direct_component = dot(vert_norm, effective_direction);
            if (direct_component < 0) {
                continue;
            }

            // TODO specular
            vec3 specular_component = vec3(0, 0, 0);

            // diffuse component
            vec3 diffuse_component = direct_component * diffuse * light_color;

            color = vec4(vec3(color) + diffuse_component + specular_component, 1);
        } else if (kind == 2) {
            // Directional light
            vec3 effective_direction = vec3(lights_data.lights[i].offset);
            float direct_component = dot(vert_norm, effective_direction);
            if (direct_component < 0) {
                continue;
            }

            // diffuse component
            vec3 diffuse_component = direct_component * diffuse * light_color;

            color = vec4(vec3(color) + diffuse_component, 1);
        } else {
            // Unknown light -- ignore!
            color = vec4(diffuse, 1);
        }
    }
    if (counts.count[1] == 0) {
        color = vec4(diffuse, 1);
    }
    // Passthrough for debug
    // color = vec4(1, 0, 0, 0);
}

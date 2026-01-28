struct Globals {
    screen_size : vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals : Globals;

struct GlyphVertexInput {
    @location(0) position : vec2<f32>,
    @location(1) offset : vec2<f32>,
    @location(2) color : vec4<f32>,
}

struct VertexInput {
    @location(0) position : vec2<f32>,
    @location(1) color : vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) color : vec4<f32>,
};

@vertex
fn glyph_vs_main(model: GlyphVertexInput) -> VertexOutput {
  var out: VertexOutput;

  let world = vec2<f32>(
    model.position.x + model.offset.x,
    model.offset.y - model.position.y
  );

  out.color = model.color;

  out.clip_position = vec4<f32>(
    (world.x / globals.screen_size[0]) * 2.0 - 1.0,
    1.0 - (world.y / globals.screen_size[1]) * 2.0,
    0.0,
    1.0
  );

  // out.clip_position = vec4<f32>(
  //   0.0, 0.0,
  //   0.0,
  //   1.0
  // );

  return out;
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  out.color = model.color;
  out.clip_position = vec4<f32>(
    model.position.x,
    model.position.y,
    0.0,
    1.0
  );

  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let rgb_color = pow((in.color.rgb + 0.055) / 1.055, vec3<f32>(2.4));

  return vec4<f32>(rgb_color, in.color.a);
}

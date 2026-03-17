struct VSOutput {
  float4 Position : SV_Position;
  float4 shaderPosition : WORLD_POSITION;
};
struct PushConstant {
  float4 origin;
};
[[vk_push_constant]]
PushConstant push_constants;
static float3 ORIGIN = float3(6., 1., -10.);
static uint[6] axis = { 0, 0, 1, 1, 2, 2 };
static float BOX_SIZE = 2.;
[shader("vertex")]
VSOutput vertexMain(float4 position) {
  var output = VSOutput();
  output.Position = position;
  output.Position.y *= -1;

  output.shaderPosition = position;
  output.shaderPosition.z = 1.;
  return output;
}
struct Ray {
  float3 origin;
  float3 direction;
};
func intersect_axis(axis: uint, direction: float3, origin: float3, at: float)
    ->float {
  return (float(at) - origin[axis]) / direction[axis];
}
func at(direction: float3, origin: float3, t: float)->float3 {
  return origin + direction * t;
}
[shader("fragment")]
func fragmentMain(float4 shaderPosition: WORLD_POSITION)->float4 {
  var direction = float3(shaderPosition.x, shaderPosition.y, 1.);
  float3 origin = float3(push_constants.origin.xyz);
  float[6] values = { intersect_axis(0, direction, origin, 0.),
                      intersect_axis(0, direction, origin, BOX_SIZE),
                      intersect_axis(1, direction, origin, 0.),
                      intersect_axis(1, direction, origin, BOX_SIZE),
                      intersect_axis(2, direction, origin, 0.),
                      intersect_axis(2, direction, origin, BOX_SIZE) };
  int min_x_index;
  int max_x_index;

  int min_y_index;
  int max_y_index;

  int min_z_index;
  int max_z_index;

  if (values[0] < values[1]) {
    min_x_index = 0;
    max_x_index = 1;
  } else {
    min_x_index = 1;
    max_x_index = 0;
  }
  if (values[2] < values[3]) {
    min_y_index = 2;
    max_y_index = 3;
  } else {
    min_y_index = 3;
    max_y_index = 2;
  }

  if (values[4] < values[5]) {
    min_z_index = 4;
    max_z_index = 5;
  } else {
    min_z_index = 5;
    max_z_index = 4;
  }

  int min_index;
  if (values[min_x_index] > values[min_y_index] &&
      values[min_x_index] > values[min_z_index]) {
    min_index = min_x_index;
  } else if (values[min_y_index] > values[min_x_index] &&
             values[min_y_index] > values[min_z_index]) {
    min_index = min_y_index;
  } else {
    min_index = min_z_index;
  }
  int max_index;
  if (values[max_x_index] < values[max_y_index] &&
      values[max_x_index] < values[max_z_index]) {
    max_index = max_x_index;
  } else if (values[max_y_index] < values[max_x_index] &&
             values[max_y_index] < values[max_z_index]) {
    max_index = max_y_index;
  } else {
    max_index = max_z_index;
  }
  if (values[max_index] >= values[min_index] && values[min_index] >= 0.) {
    var position = at(direction, origin, values[min_index]);
    var block_position = floor(position);
    var color = block_position / BOX_SIZE;
    return float4(color, 1.);

  } else {
    return float4(0., 0., 0.2, 0.0);
  }
}

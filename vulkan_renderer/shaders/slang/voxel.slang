struct VSOutput {
  float4 Position : SV_Position;
  float4 shaderPosition : WORLD_POSITION;
};
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
func intersect_axis(axis: uint, direction: float3, origin: float3, at: float)
    ->float {
  return (float(at) - origin[axis]) / direction[axis];
}
[shader("fragment")]
func fragmentMain(float4 shaderPosition: WORLD_POSITION)->float4 {
  var direction = float3(shaderPosition.x, shaderPosition.y, 1.);
  float[6] values = { intersect_axis(0, direction, ORIGIN, 0.),
                      intersect_axis(0, direction, ORIGIN, BOX_SIZE),
                      intersect_axis(1, direction, ORIGIN, 0.),
                      intersect_axis(1, direction, ORIGIN, BOX_SIZE),
                      intersect_axis(2, direction, ORIGIN, 0.),
                      intersect_axis(2, direction, ORIGIN, BOX_SIZE) };
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

    return float4(1., 1., 1., 1, );
  } else {
    return float4(0., 0., 0.2, 1.);
  }
  // var box_min = float3(0., 0., 0.);
  // var box_max = float3(4., 4., 4.);
  //
  // bool in_box = rayDirection.x > box_min.x && rayDirection.x < box_max.x &&
  //              rayDirection.y > box_min.y && rayDirection.y < box_max.y;
  // if (in_box) {
  //  return float4(1., 1., 1., 1, );
  //} else {
  //  return float4(1., 0., 0., 1, );
  //}
}

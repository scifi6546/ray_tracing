SamplerState sample_color;
Texture2D texture;
struct VSOutput {
  float4 Position : SV_Position;
  float4 color : VERTEX_COLOR;
  float2 uv : TEXTURE_COORDINATES;
};
[shader("vertex")]
VSOutput vertexMain(float4 position, float4 color, float2 uv) {
  var output = VSOutput();
  output.Position = position;
  output.color = color;
  output.uv = uv;
  return output;
}
[shader("fragment")]
float4 fragmentMain(float4 color: VERTEX_COLOR,
                    float2 uv: TEXTURE_COORDINATES) {

  return texture.Sample(sample_color, uv) * color;
}

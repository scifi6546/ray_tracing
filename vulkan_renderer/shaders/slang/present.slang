SamplerState sample_color;
Texture2D texture;

[shader("fragment")]
float4 fragmentMain(float4 color, float2 uv) {
  var color = texture.Sample(sample_color, uv);
  return color;
}

struct VSOutput
{
    float4 Position : SV_Position;
    float4 shaderPosition : WORLD_POSITION;
};
[shader("vertex")]
VSOutput vertexMain(float4 position)
{
    var output = VSOutput();
    output.Position = position;
    output.shaderPosition = position;
    return output;
}
[shader("fragment")]
float4 fragmentMain(float4 rayDirection: WORLD_POSITION)
{
    float2 box_min = float2(-0.5, -0.5);
    float2 box_max = float2(0.5, 0.5);
    bool in_box = rayDirection.x > box_min.x &&
                  rayDirection.x < box_max.x &&
                  rayDirection.y > box_min.y &&
                  rayDirection.y < box_max.y;
    if (in_box)
    {
        return float4(1., 1., 1., 1, );
    }
    else
    {
        return float4(1., 0., 0., 1, );
    }
}

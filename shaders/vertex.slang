// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

module vertex;

public struct Vertex {
    public float4 pos : SV_Position;
    public float4 color;
    public float4 normal;
    public float2 uv;
};

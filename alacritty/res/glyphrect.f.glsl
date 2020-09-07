// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#version 300 es
precision mediump float;

uniform sampler2D atlas;

smooth in vec2 uv;
flat in vec3 fg, bg;

out vec4 FragColor;

void main()
{
		vec4 mask = texture(atlas, uv);

	// if (colored)
	// 	return mix(bg, mask.rgb, mask.a);
	// else
	// 	return mix(bg, fg.rgb, mask.rgb);

		// FIXME pass colored bit
		bool colored = false;
		if (colored) {
			FragColor = mask;
		} else {
			FragColor = vec4(fg, mask.r);
		}
}

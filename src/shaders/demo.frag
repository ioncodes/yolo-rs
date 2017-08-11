#version 140

uniform float time;
uniform vec2 mouse;
uniform vec2 resolution;
out vec4 fragColor;

#define TAU 6.283185307179586

#define PI TAU/2.0
#define PI_2 (PI/2.0)
#define LINEWIDTH (PI/36.0)
#define NOTLINE (PI_2 - LINEWIDTH)
#define EDGEWIDTH ((LINEWIDTH) * 2.5)
#define NOTEDGE (NOTLINE - ((EDGEWIDTH) * 2.0))

#define COLOR_LINE   vec3(0.7803921568627451, 0.9568627450980393, 0.39215686274509803)
#define COLOR_A_EDGE vec3(1.0, 0.4196078431372549, 0.4196078431372549)
#define COLOR_A_MID  vec3(0.7686274509803922, 0.30196078431372547, 0.34509803921568627)
#define COLOR_B_EDGE vec3(0.3058823529411765, 0.803921568627451, 0.7686274509803922)
#define COLOR_B_MID  vec3(0.3333333333333333, 0.3843137254901961, 0.4392156862745098)

#define FADETIME 4.5

#define atan2(y,x) ((abs(x) > abs(y)) ? (3.14159265358979/2.0 - atan(x,y)) : atan(y,x))

vec3 colorize(in float t, in vec3 edge, in vec3 mid) {
	if (t > NOTLINE) {
		return COLOR_LINE;
	} else {
		if ((t < EDGEWIDTH) || (t > (EDGEWIDTH + NOTEDGE))) {
			return edge;
		} else {
			return mid;
		}
	}
}

#define POWMIN   (-0.25)
#define POWRANGE 1.4
#define POWBASE  (POWRANGE + POWMIN)


void main() {
	vec2 uv = gl_FragCoord.xy / resolution.xy;
	vec2 position = (uv * 2.0) - 1.0;
	position.y *= resolution.y/resolution.x;
	vec3 color = vec3(0.0);

	float fade = sin(time * FADETIME);

	float theta = atan(position.y/ position.x);
	float r = length(position) + 0.2;
	r += mouse.y;
	float warptime = time / 5.3;

	float cscale = 1.2 + (0.5 * mouse.x);

	float warp = sin((r + warptime) * 7.0) / (cscale - cos(theta));
	warp -= theta;

	float tt = mod(time - warp, TAU/1.0);
	tt = mod(tt, PI_2) * 2.0;
	
	color = (tt > PI_2)
		? colorize(tt - PI_2, COLOR_A_EDGE, COLOR_A_MID)
		: colorize(tt,        COLOR_B_EDGE, COLOR_B_MID);


	fragColor = vec4(color, 1.0);
}
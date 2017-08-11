#version 140
in vec2 position;
void main() {
	float z = 0.0;
	float x = position.x;
	float y = position.y;
    gl_Position = vec4(x, y, z, 1.0);
}
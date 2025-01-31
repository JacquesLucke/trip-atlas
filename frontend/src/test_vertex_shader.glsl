precision highp float;

attribute vec2 aVertexPosition;
attribute float aVertexTime;

uniform vec2 mapCenter;
uniform vec2 mapExtent;

varying float v_time;

void main() {
  gl_Position = vec4((aVertexPosition - mapCenter) / mapExtent * 2.0, 0.0, 1.0);
  gl_PointSize = 10.0;
  v_time = aVertexTime;
}

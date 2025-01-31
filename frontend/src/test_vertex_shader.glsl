precision highp float;

attribute vec2 aVertexPosition;

uniform vec2 mapCenter;
uniform vec2 mapExtent;

void main() {
  gl_Position = vec4((aVertexPosition - mapCenter) / mapExtent * 2.0, 0.0, 1.0);
  gl_PointSize = 10.0;
}

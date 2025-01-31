precision highp float;

attribute vec2 quadOffsetAttr;
attribute vec2 locationAttr;
attribute float timeAttr;

uniform vec2 mapCenter;
uniform vec2 mapExtent;
uniform vec2 resolution;
uniform float stationSize;

varying float v_time;
varying vec2 v_quad;

void main() {
  float aspect = resolution.x / resolution.y;
  vec2 quadOffset =
      quadOffsetAttr * vec2(1.0, aspect) * (stationSize / resolution.x);
  gl_Position =
      vec4((locationAttr - mapCenter) / mapExtent * 2.0 + quadOffset, 0.0, 1.0);
  v_time = timeAttr;
  v_quad = quadOffsetAttr;
}

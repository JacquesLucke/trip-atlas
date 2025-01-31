precision highp float;

attribute vec2 quadOffsetAttr;
attribute vec2 locationAttr;
attribute float timeAttr;

uniform vec2 mapCenter;
uniform vec2 mapExtent;
uniform vec2 resolution;

varying float v_time;

void main() {
  float aspect = resolution.x / resolution.y;
  gl_Position = vec4((locationAttr - mapCenter) / mapExtent * 2.0 +
                         quadOffsetAttr * vec2(1.0, aspect) * 0.002,
                     0.0, 1.0);
  gl_PointSize = 10.0;
  v_time = timeAttr;
}

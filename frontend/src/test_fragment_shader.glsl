precision mediump float;

varying float v_time;
varying vec2 v_quad;

uniform float borderThickness;

void main() {
  float distToCenter = length(v_quad);
  if (distToCenter > 1.0) {
    discard;
  }

  if (distToCenter > 1.0 - borderThickness) {
    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    float time_factor = v_time / 5000.0;
    gl_FragColor = vec4(time_factor, 1.0 - time_factor, 0.5 - time_factor, 1.0);
  }
}

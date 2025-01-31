precision mediump float;

varying float v_time;

void main() {
  float time_factor = v_time / 10000.0;
  gl_FragColor = vec4(time_factor, 1.0 - time_factor, 0.5 - time_factor, 1.0);
}

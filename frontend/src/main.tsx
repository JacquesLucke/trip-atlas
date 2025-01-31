import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./tailwind.css";
import * as L from "leaflet";
import "leaflet/dist/leaflet.css";
import staticLocations from "./stations_test_data.json";

const useStaticLocations = false;
const locationsUrl =
  "https://trip-atlas.fsn1.your-objectstorage.com/test-data/stations_test_data.json";

interface SourceJson {
  stations: StationInfo[];
}

interface StationInfo {
  latitude: number;
  longitude: number;
  name: string;
  time?: number;
}

async function main() {
  let locations = (
    useStaticLocations
      ? staticLocations
      : await (await fetch(locationsUrl)).json()
  ) as SourceJson;

  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>
  );

  // Initialize map.
  const mapContainer = document.getElementById("map-container")!;
  const defaultCoordinates = new L.LatLng(52.637778, 13.203611);
  const defaultZoom = 14;
  const savedMapView = JSON.parse(localStorage.getItem("map-view") ?? "{}");

  const map = L.map(mapContainer, {
    // Fractional zoom has visible lines between the tiles currently.
    // https://github.com/Leaflet/Leaflet/issues/3575
    zoomSnap: 0,
    // zoomDelta: 1,
    zoomAnimation: false,
  }).setView(
    new L.LatLng(
      savedMapView.latitude ?? defaultCoordinates.lat,
      savedMapView.longitude ?? defaultCoordinates.lng
    ),
    savedMapView.zoom ?? defaultZoom
  );

  map.on("moveend", () => {
    const center = map.getCenter();
    const zoom = map.getZoom();
    localStorage.setItem(
      "map-view",
      JSON.stringify({ latitude: center.lat, longitude: center.lng, zoom })
    );
  });

  // Add the Leaflet specific attribution.
  map.attributionControl.setPrefix(
    "<a href='https://leafletjs.com/'>Leaflet</a>"
  );

  // Add the OpenStreetMap background layer.
  L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    attribution:
      '<a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  }).addTo(map);

  await addOverlayCanvas(map, locations);
}

import vertexShaderSrc from "./test_vertex_shader.glsl";
import fragmentShaderSrc from "./test_fragment_shader.glsl";

async function addOverlayCanvas(map: L.Map, locations: SourceJson) {
  const canvas = document.getElementById(
    "map-container-overlay"
  )! as HTMLCanvasElement;

  const gl = canvas.getContext("webgl2")!;

  const program = createShaderProgram(gl, vertexShaderSrc, fragmentShaderSrc)!;
  const attrs = {
    quadOffset: gl.getAttribLocation(program, "quadOffsetAttr"),
    locations: gl.getAttribLocation(program, "locationAttr"),
    times: gl.getAttribLocation(program, "timeAttr"),
  };

  const quadOffsets = new Float32Array([
    -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
  ]);

  const positions = new Float32Array(locations.stations.length * 2);
  const times = new Float32Array(locations.stations.length);
  for (let i = 0; i < locations.stations.length; i++) {
    const station = locations.stations[i];
    positions[i * 2] = station.longitude;
    positions[i * 2 + 1] = station.latitude;
    times[i] = station.time ?? Number.MAX_VALUE;
  }

  const locationsAttrBuffer = gl.createBuffer()!;
  gl.bindBuffer(gl.ARRAY_BUFFER, locationsAttrBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

  const timesAttrBuffer = gl.createBuffer()!;
  gl.bindBuffer(gl.ARRAY_BUFFER, timesAttrBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, times, gl.STATIC_DRAW);

  const quadOffsetAttrBuffer = gl.createBuffer()!;
  gl.bindBuffer(gl.ARRAY_BUFFER, quadOffsetAttrBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, quadOffsets, gl.STATIC_DRAW);

  function render() {
    const mapSize = map.getSize();
    const mapCenter = map.getCenter();
    const mapBounds = map.getBounds();
    const mapZoom = map.getZoom();
    canvas.width = mapSize.x;
    canvas.height = mapSize.y;

    gl.clearColor(0, 0, 0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.viewport(0, 0, mapSize.x, mapSize.y);

    gl.useProgram(program);

    gl.bindBuffer(gl.ARRAY_BUFFER, locationsAttrBuffer);
    gl.vertexAttribPointer(attrs.locations, 2, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(attrs.locations);
    gl.vertexAttribDivisor(attrs.locations, 1);

    gl.bindBuffer(gl.ARRAY_BUFFER, timesAttrBuffer);
    gl.vertexAttribPointer(attrs.times, 1, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(attrs.times);
    gl.vertexAttribDivisor(attrs.times, 1);

    gl.bindBuffer(gl.ARRAY_BUFFER, quadOffsetAttrBuffer);
    gl.vertexAttribPointer(attrs.quadOffset, 2, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(attrs.quadOffset);

    gl.uniform2f(
      gl.getUniformLocation(program, "mapCenter"),
      mapCenter.lng,
      mapCenter.lat
    );
    gl.uniform2f(
      gl.getUniformLocation(program, "mapExtent"),
      mapBounds.getEast() - mapBounds.getWest(),
      mapBounds.getNorth() - mapBounds.getSouth()
    );
    gl.uniform2f(
      gl.getUniformLocation(program, "resolution"),
      mapSize.x,
      mapSize.y
    );
    gl.uniform1f(
      gl.getUniformLocation(program, "borderThickness"),
      mapZoom >= 12 ? 0.2 : 0.0
    );

    const sizeByZoom = new Map<number, number>();
    sizeByZoom.set(18, 20.0);
    sizeByZoom.set(17, 19.0);
    sizeByZoom.set(16, 17.0);
    sizeByZoom.set(15, 15.0);
    sizeByZoom.set(14, 12.0);
    sizeByZoom.set(13, 10.0);
    sizeByZoom.set(12, 8.0);
    sizeByZoom.set(11, 6.0);
    sizeByZoom.set(10, 4.0);
    sizeByZoom.set(9, 3.0);
    sizeByZoom.set(8, 3.0);
    gl.uniform1f(
      gl.getUniformLocation(program, "stationSize"),
      sizeByZoom.get(Math.round(mapZoom)) ?? 2.0
    );

    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, locations.stations.length);
  }

  render();

  map.on("move", render);
  map.on("zoomlevelschange", render);
  map.on("zoomanim", render);
  map.on("drag", render);
}

function createShaderProgram(
  gl: WebGLRenderingContext,
  vertexShaderSrc: string,
  fragmentShaderSrc: string
) {
  const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vertexShaderSrc);
  const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSrc);

  const program = gl.createProgram()!;
  gl.attachShader(program, vertexShader!);
  gl.attachShader(program, fragmentShader!);
  gl.linkProgram(program);

  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error(gl.getProgramInfoLog(program));
    return null;
  }
  return program;
}

function loadShader(gl: WebGLRenderingContext, type: GLenum, source: string) {
  const shader = gl.createShader(type);
  if (!shader) {
    return null;
  }
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  const success = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  if (success) {
    return shader;
  }
  console.error(gl.getShaderInfoLog(shader));
  gl.deleteShader(shader);
  return null;
}

main();

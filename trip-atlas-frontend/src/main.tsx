import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./tailwind.css";
import * as L from "leaflet";
import "leaflet/dist/leaflet.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);

// Initialize map.
const mapContainerId = "map-container";
const defaultCoordinates = new L.LatLng(52.637778, 13.203611);
const defaultZoom = 14;
const map = L.map(mapContainerId, {
  // Fractional zoom has visible lines between the tiles currently.
  // https://github.com/Leaflet/Leaflet/issues/3575
  // zoomSnap: 0,
  // zoomDelta: 1,
}).setView(defaultCoordinates, defaultZoom);

// Add the Leaflet specific attribution.
map.attributionControl.setPrefix(
  "<a href='https://leafletjs.com/'>Leaflet</a>"
);

// Add the OpenStreetMap background layer.
L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
  attribution:
    '<a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

interface TileOverlayInfo {}

const myPoints = [new L.LatLng(52.637778, 13.203611), new L.LatLng(52, 13)];

const activeTiles = new Map<HTMLElement, TileOverlayInfo>();

const CustomMapLayer = L.GridLayer.extend({
  createTile: function (this: L.GridLayer, coords: L.Coords) {
    const tileSize = this.getTileSize();
    const corner1 = map.unproject(coords.scaleBy(tileSize), coords.z);
    const corner2 = map.unproject(
      coords.add([1, 1]).scaleBy(tileSize),
      coords.z
    );
    const bounds = new L.LatLngBounds([corner1, corner2]);

    const tile = L.DomUtil.create("div", "leaflet-tile");
    const svgNS = "http://www.w3.org/2000/svg";
    const svg = document.createElementNS(svgNS, "svg");
    svg.setAttribute("width", `${tileSize.x}px`);
    svg.setAttribute("height", `${tileSize.y}px`);

    for (const point of myPoints) {
      const pixelPos = map
        .project(point, coords.z)
        .subtract(coords.scaleBy(tileSize));

      const circle = document.createElementNS(svgNS, "circle");
      circle.setAttribute("cx", `${pixelPos.x}`);
      circle.setAttribute("cy", `${pixelPos.y}`);
      circle.setAttribute("r", `10px`);
      circle.setAttribute("fill", "red");
      circle.setAttribute("opacity", "1");

      svg.appendChild(circle);
    }

    tile.appendChild(svg);
    activeTiles.set(tile, {});
    return tile;
  },
});
const customLayer = new CustomMapLayer();
customLayer.on("tileunload", (event: L.TileEvent) => {
  activeTiles.delete(event.tile);
  console.log(activeTiles.size);
});
customLayer.addTo(map);

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
const map = L.map(mapContainerId).setView(defaultCoordinates, defaultZoom);

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

const activeTiles = new Map<HTMLElement, TileOverlayInfo>();

const CustomMapLayer = L.GridLayer.extend({
  createTile: (coords: L.Point) => {
    const tile = L.DomUtil.create("div", "leaflet-tile");
    tile.innerHTML = `<div class="bg-white w-fit">${coords}</div>`;

    activeTiles.set(tile, {});
    console.log(activeTiles.size);
    return tile;
  },
});
const customLayer = new CustomMapLayer();
customLayer.on("tileunload", (event: L.TileEvent) => {
  activeTiles.delete(event.tile);
  console.log(activeTiles.size);
});
customLayer.addTo(map);

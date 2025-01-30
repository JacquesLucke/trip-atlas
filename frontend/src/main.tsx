import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App.tsx";
import "./tailwind.css";
import * as L from "leaflet";
import "leaflet/dist/leaflet.css";
import staticLocations from "./stations_test_data.json";
import RBush from "rbush";

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
  const mapContainerId = "map-container";
  const defaultCoordinates = new L.LatLng(52.637778, 13.203611);
  const defaultZoom = 14;
  const savedMapView = JSON.parse(localStorage.getItem("map-view") ?? "{}");

  const map = L.map(mapContainerId, {
    // Fractional zoom has visible lines between the tiles currently.
    // https://github.com/Leaflet/Leaflet/issues/3575
    // zoomSnap: 0,
    // zoomDelta: 1,
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

  interface TileOverlayInfo {}

  const stationsTree = new RBush<StationInfo>();

  stationsTree.load(
    locations.stations.map((station) => {
      return {
        ...station,
        minX: station.longitude,
        minY: station.latitude,
        maxX: station.longitude,
        maxY: station.latitude,
      };
    })
  );

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

      const circleRadius = 3;

      const paddedBounds = bounds.pad(0.3);

      const foundStations = stationsTree.search({
        minX: paddedBounds.getWest(),
        minY: paddedBounds.getSouth(),
        maxX: paddedBounds.getEast(),
        maxY: paddedBounds.getNorth(),
      });

      for (const station of foundStations) {
        const pixelPos = map
          .project(new L.LatLng(station.latitude, station.longitude), coords.z)
          .subtract(coords.scaleBy(tileSize));
        if (
          pixelPos.x < -circleRadius ||
          pixelPos.x > tileSize.x + circleRadius ||
          pixelPos.y < -circleRadius ||
          pixelPos.y > tileSize.y + circleRadius
        ) {
          continue;
        }

        const time = station.time ?? Number.MAX_VALUE;

        const circle = document.createElementNS(svgNS, "circle");
        circle.setAttribute("cx", `${pixelPos.x}`);
        circle.setAttribute("cy", `${pixelPos.y}`);
        circle.setAttribute("r", `${circleRadius}px`);
        circle.setAttribute(
          "fill",
          `hsl(${Math.min(1, time / 3000)}turn, 100%, 50%)`
        );
        circle.setAttribute("opacity", `1.0`);

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
  });
  customLayer.addTo(map);
}

main();

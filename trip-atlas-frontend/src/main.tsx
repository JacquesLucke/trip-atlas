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

const map = L.map("map-container").setView([52.637778, 13.203611], 14);
map.attributionControl.setPrefix(
  "<a href='https://leafletjs.com/'>Leaflet</a>"
);

L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
  attribution:
    '<a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

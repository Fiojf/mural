// Render SVG icons → PNGs with proper transparency.
// Uses @resvg/resvg-js (installed via npm install --no-save).

import { readFileSync, writeFileSync } from "node:fs";
import { Resvg } from "@resvg/resvg-js";

function render(svgPath, pngPath, width) {
  const svg = readFileSync(svgPath);
  const r = new Resvg(svg, { fitTo: { mode: "width", value: width } });
  const png = r.render().asPng();
  writeFileSync(pngPath, png);
  console.log(`${pngPath} (${width}px)`);
}

render("src-tauri/icons/icon.svg", "src-tauri/icons/icon.png", 1024);
render("src-tauri/icons/tray.svg", "src-tauri/icons/tray.png", 44);

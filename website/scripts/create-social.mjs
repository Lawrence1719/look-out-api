import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { Resvg } from "@resvg/resvg-js";

const svg = await readFile(
  new URL("../public/social.svg", import.meta.url),
  "utf8",
);
const image = new Resvg(svg, {
  font: {
    loadSystemFonts: false,
    fontFiles: [fileURLToPath(new URL("./fonts/Manrope.ttf", import.meta.url))],
    defaultFontFamily: "Manrope",
    sansSerifFamily: "Manrope",
  },
});
await writeFile(
  new URL("../public/social.png", import.meta.url),
  image.render().asPng(),
);

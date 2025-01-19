import * as toml from "jsr:@std/toml";
import { XMLParser } from "npm:fast-xml-parser";
import { Database } from "jsr:@db/sqlite@0.12";
import * as path from "jsr:@std/path";

const data_dir = "./data";
const stations_xml_path = path.join(data_dir, "stations.xml");
const stations_db_path = path.join(data_dir, "stations.db");
Deno.mkdirSync(data_dir, { recursive: true });

const secrets = toml.parse(Deno.readTextFileSync("./secrets.toml"));

async function main() {
  const command = Deno.args[0];
  if (command === "db-stations") {
    await load_stations();
  }
}

async function load_stations() {
  const stations_xml = await get_or_load_db_stations_xml();

  const parser = new XMLParser({ ignoreAttributes: false });
  const parsed_stations = parser.parse(stations_xml);

  await removeFileIfExists(stations_db_path);
  const db = new Database(stations_db_path);
  db.run(
    `CREATE TABLE stations (
        meta TEXT,
        name TEXT NOT NULL,
        eva TEXT NOT NULL,
        ds100 TEXT NOT NULL,
        db BOOLEAN NOT NULL,
        creationts TEXT NOT NULL
    );`
  );
  db.transaction(() => {
    const insert = db.prepare(
      `INSERT INTO stations (meta, name, eva, ds100, db, creationts) VALUES (?, ?, ?, ?, ?, ?)`
    );
    for (const station of parsed_stations.stations.station) {
      console.log(station);
      insert.run([
        station["@_meta"] ?? null,
        station["@_name"],
        station["@_eva"],
        station["@_ds100"],
        station["@_db"] === "true",
        station["@_creationts"],
      ]);
    }
  })();
  db.close();

  //   await Deno.writeTextFile(`${data_dir}/stations.xml`, stations_xml);
}

async function get_or_load_db_stations_xml() {
  try {
    return await Deno.readTextFile(stations_xml_path);
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) {
      return await fetch(
        "https://apis.deutschebahn.com/db-api-marketplace/apis/timetables/v1/station/*",
        {
          headers: {
            "DB-Client-ID": secrets["db_client_id"] as string,
            "DB-API-Key": secrets["db_client_api_key"] as string,
            accept: "application/xml",
          },
        }
      ).then(async (response) => {
        const stations_xml = await response.text();
        await Deno.writeTextFile(stations_xml_path, stations_xml);
        return stations_xml;
      });
    } else {
      throw e;
    }
  }
}

async function removeFileIfExists(path: string) {
  try {
    await Deno.remove(path, {});
  } catch (e) {
    if (e instanceof Deno.errors.NotFound) {
      // ignore
    } else {
      throw e;
    }
  }
}

main();

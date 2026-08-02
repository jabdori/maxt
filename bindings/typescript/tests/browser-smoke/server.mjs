import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../..", import.meta.url));
const contentTypes = new Map([
  [".js", "text/javascript; charset=utf-8"],
  [".wasm", "application/wasm"],
]);

createServer(async (request, response) => {
  const pathname = new URL(request.url ?? "/", "http://127.0.0.1").pathname;
  if (pathname === "/health") {
    response.writeHead(204).end();
    return;
  }
  if (pathname === "/") {
    response.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    response.end("<!doctype html><meta charset=utf-8>");
    return;
  }

  const file = resolve(root, `.${decodeURIComponent(pathname)}`);
  const allowed = [resolve(root, "dist"), resolve(root, "wasm")]
    .some((directory) => file.startsWith(`${directory}${sep}`));
  if (!allowed) {
    response.writeHead(404).end();
    return;
  }
  try {
    if (!(await stat(file)).isFile()) throw new Error("not a file");
    response.writeHead(200, {
      "content-type": contentTypes.get(extname(file)) ?? "application/octet-stream",
    });
    createReadStream(file).pipe(response);
  } catch {
    response.writeHead(404).end();
  }
}).listen(4173, "127.0.0.1");

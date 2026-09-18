import assert from "node:assert/strict";
import { test } from "node:test";
import { build } from "esbuild";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const dir = await mkdtemp(join(tmpdir(), "opennex-layout-"));
const root = new URL("../", import.meta.url).pathname;
await build({
  stdin: {
    contents: 'export * from "./src/dock/model"; export * from "./src/dock/mainLayout"; export * from "./src/workspaces"; export * from "./src/terminal/registry"; export { Actions, DockLocation } from "flexlayout-react";',
    resolveDir: root,
  },
  outfile: join(dir, "layout.mjs"), bundle: true, platform: "node", format: "esm",
});
const api = await import(pathToFileURL(join(dir, "layout.mjs")));
await rm(dir, { recursive: true });
const values = new Map();
globalThis.localStorage = {
  getItem: key => values.get(key) ?? null,
  setItem: (key, value) => values.set(key, value),
};

function fixture() {
  values.clear();
  const outer = api.mainModelFrom(undefined).toJson();
  outer.layout.children[0].weight = 31;
  outer.layout.children[1].weight = 69;
  const other = structuredClone(outer);
  other.layout.children[0].weight = 80;
  const one = api.newTermJson();
  const term = api.termModelFrom(api.newTermJson(), api.nextSlot);
  const second = api.newTermJson().layout.children[0].children[0];
  term.doAction(api.Actions.addNode(second, api.termTabsetId(term), api.DockLocation.RIGHT, -1));
  const two = term.toJson();
  localStorage.setItem("opennex-workspaces", JSON.stringify([
    { name: "A", mainJson: outer, termJson: one },
    { name: "B", mainJson: other, termJson: two },
  ]));
  return { outer, other, one, two };
}

test("legacy layout migrates once; workspace persistence cannot replace global layout", () => {
  fixture();
  const main = api.loadMainModel();
  assert.equal(main.toJson().layout.children[0].weight, 31);
  api.persistMainModel(main);
  const saved = localStorage.getItem("opennex-main-layout");
  const workspaces = api.loadWorkspaces();
  assert.ok(workspaces.every(w => !("mainJson" in w)));
  api.persistWorkspaces(workspaces);
  for (const workspace of [workspaces[0], workspaces[1], workspaces[0]]) {
    const term = api.termModelFrom(workspace.termJson, api.nextSlot);
    const slots = api.collectModelTermSlots(term);
    console.log(JSON.stringify({ workspace: workspace.name, terminalSlots: slots,
      globalNavWeight: api.loadMainModel().toJson().layout.children[0].weight,
      workspaceHasMainJson: "mainJson" in workspace }));
    assert.equal(slots.length, workspace.name === "A" ? 1 : 2);
    assert.equal(localStorage.getItem("opennex-main-layout"), saved);
    assert.deepEqual(api.loadMainModel().toJson(), main.toJson());
  }
});

test("global layout wins over conflicting legacy snapshots on restart", () => {
  const { other } = fixture();
  api.persistMainModel(api.mainModelFrom(other));
  assert.equal(api.loadMainModel().toJson().layout.children[0].weight, 80);
});

test("corrupt storage falls back to a usable layout", () => {
  fixture();
  localStorage.setItem("opennex-main-layout", "{");
  assert.equal(api.loadMainModel().toJson().layout.children[0].weight, 31);
  localStorage.setItem("opennex-workspaces", "{");
  assert.ok(api.loadMainModel().getNodeById("nav"));
});

test("legacy templates restore only terminal splits with fresh slots and paths", () => {
  const { other, two } = fixture();
  const originalSlots = api.jsonTermSlots(two);
  const cwdMap = Object.fromEntries(originalSlots.map(slot => [slot, `/project/${slot}`]));
  localStorage.setItem("opennex-templates", JSON.stringify([
    { id: "tpl", name: "template", mainJson: other, termJson: two, cwdMap },
  ]));
  const [template] = api.loadTemplates();
  assert.equal("mainJson" in template, false);
  const { json } = api.renumberTermJson(api.withCwd(template.termJson, template.cwdMap));
  const newSlots = api.jsonTermSlots(json);
  assert.equal(newSlots.length, 2);
  assert.ok(newSlots.every(slot => !originalSlots.includes(slot)));
  assert.deepEqual(Object.values(api.cwdMapFromJson(json)), Object.values(cwdMap));
  assert.deepEqual(template.termJson, two);
  assert.deepEqual(json.layout.children.map(n => n.weight), two.layout.children.map(n => n.weight));
});

test("last terminal cannot close; adding and removing restores close affordances", () => {
  const model = api.termModelFrom(api.newTermJson(), api.nextSlot);
  const id = `term-${api.collectModelTermSlots(model)[0]}`;
  assert.equal(api.canCloseTerminal(model, id), false);
  assert.equal(model.getNodeById(id).isEnableClose(), false);
  const second = api.newTermJson().layout.children[0].children[0];
  model.doAction(api.Actions.addNode(second, api.termTabsetId(model), api.DockLocation.CENTER, -1));
  assert.equal(api.canCloseTerminal(model, id), true);
  assert.equal(model.getNodeById(id).isEnableClose(), true);
  model.doAction(api.Actions.deleteTab(second.id));
  assert.equal(api.canCloseTerminal(model, id), false);
  assert.equal(model.getNodeById(id).isEnableClose(), false);
});

test("broadcast rejects non-members and excludes closed sessions", () => {
  globalThis.WebSocket = { OPEN: 1 };
  const sent = [];
  for (const slot of [1, 2, 3]) api.registerSocket(slot, { readyState: 1, send: () => sent.push(slot) });
  api.broadcastEnabled.value = true;
  api.broadcastGroup.add(1); api.broadcastGroup.add(2);
  assert.equal(api.broadcastInput(3, new Uint8Array([65])), false);
  api.broadcastInput(1, new Uint8Array([65]));
  assert.deepEqual(sent, [2]);
  api.broadcastGroup.delete(2);
  assert.equal(api.broadcastInput(1, new Uint8Array([65])), false);
  api.broadcastEnabled.value = false; api.broadcastGroup.clear(); api.sockets.clear();
});

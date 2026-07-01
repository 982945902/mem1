// Unit tests for the async client: mock global fetch, assert request shape + parsing.
// Run: npm run build && node --test dist/src/*.test.js

import { test } from "node:test";
import assert from "node:assert/strict";
import { Memory } from "./memory.js";
import { Mem1Client } from "./client.js";

type Captured = { url: string; method: string; body: unknown };

function mockFetch(status: number, json: unknown): { calls: Captured[] } {
  const calls: Captured[] = [];
  (globalThis as unknown as { fetch: unknown }).fetch = async (
    url: URL | string,
    init: { method?: string; body?: string } = {},
  ) => {
    calls.push({
      url: url.toString(),
      method: init.method ?? "GET",
      body: init.body ? JSON.parse(init.body) : undefined,
    });
    return {
      status,
      headers: new Map([["content-type", "application/json"]]) as unknown as Headers,
      json: async () => json,
      text: async () => JSON.stringify(json),
    } as unknown as Response;
  };
  return { calls };
}

test("client.add posts content and parses AddResponse", async () => {
  const { calls } = mockFetch(201, { results: [{ id: "m1", content: "x", user_id: "u", metadata: {}, created_at: "t" }] });
  const c = new Mem1Client({ baseUrl: "http://h:1" });
  const r = await c.add("u", "hello", { scope: "p" });
  assert.equal(r.results[0].id, "m1");
  assert.equal(calls[0].method, "POST");
  assert.ok(calls[0].url.endsWith("/memories"));
  assert.deepEqual(calls[0].body, { user_id: "u", content: "hello", metadata: { scope: "p" } });
});

test("Memory.add with messages normalizes and forwards", async () => {
  const { calls } = mockFetch(201, { results: [{ id: "m1", content: "x", user_id: "u", metadata: {}, created_at: "t" }] });
  const m = new Memory({ baseUrl: "http://h:1" });
  await m.add([{ role: "user", content: "hi" }], "u", { agent_id: "a" });
  assert.deepEqual(calls[0].body, {
    user_id: "u",
    messages: [{ role: "user", content: "hi" }],
    metadata: { agent_id: "a" },
  });
});

test("Memory.search forwards filters", async () => {
  const { calls } = mockFetch(200, { results: [], formatted_context: "ctx" });
  const m = new Memory({ baseUrl: "http://h:1" });
  const r = await m.search("q", "u", 5, { run_id: "r1" });
  assert.equal(r.formatted_context, "ctx");
  assert.deepEqual(calls[0].body, { user_id: "u", query: "q", limit: 5, filters: { run_id: "r1" } });
});

test("client.get returns null on 404", async () => {
  mockFetch(404, {});
  const c = new Mem1Client({ baseUrl: "http://h:1" });
  assert.equal(await c.get("nope", "u"), null);
});

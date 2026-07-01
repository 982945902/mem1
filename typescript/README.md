# mem1 — TypeScript SDK

Async TypeScript/JavaScript client for the [mem1](../README.md) AI memory
service. All methods return Promises. Uses native `fetch` (Node ≥ 18, Deno,
Bun, browsers), zero runtime dependencies.

## Install

```bash
npm install mem1        # once published
# or, from this repo:
cd typescript && npm install && npm run build
```

## Usage

```ts
import { Memory } from "mem1";

const memory = new Memory({ baseUrl: "http://127.0.0.1:8080" });

// Store — a string or a conversation
await memory.add("Alice prefers tea over coffee.", "alice");
await memory.add(
  [{ role: "user", content: "I work on NLP and like Rust." }],
  "alice",
  { agent_id: "a1" }, // optional metadata filters (scope / agent_id / run_id ...)
);

// Search — returns matches + a prompt-ready formatted_context
const res = await memory.search("what does alice drink?", "alice", 5);
console.log(res.formatted_context, res.results);

// Manage
await memory.get(id, "alice");
await memory.update(id, "new content", "alice");
await memory.delete(id, "alice");
await memory.getAll("alice");
await memory.history(id, "alice");
```

### Low-level client

`Mem1Client` mirrors the HTTP API one-to-one (`add`, `addMessages`, `search`,
`list`, `get`, `update`, `delete`, `deleteAll`, `history`, `users`, `reset`);
`Memory` is the mem0-style convenience layer on top.

```ts
import { Mem1Client, ClientError } from "mem1";
const client = new Mem1Client({ baseUrl: "http://127.0.0.1:8080", apiKey: "..." });
```

Errors throw `ClientError` (`.code`, `.message`, `.traceId`).

## Develop

```bash
npm install
npm run build     # tsc -> dist/
npm test          # node --test on compiled tests
npm run example   # runs examples/example.ts against a local server
```

Licensed [Apache-2.0](../LICENSE).

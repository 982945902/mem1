// mem1 TypeScript SDK example (async) — run against a local mem1-server.
//
// Prerequisites: start mem1-server (cd mem1-server && cargo run)
// Build + run:   npm install && npm run build && npm run example

import { Memory } from "../src/index.js";

async function main(): Promise<void> {
  const userId = "alice";
  const memory = new Memory({ baseUrl: "http://127.0.0.1:8080" });

  console.log("1. Add text memory");
  const r = await memory.add("Alice prefers dark mode and uses Python for ML.", userId);
  const firstId = r.results[0].id;
  console.log("   Created id:", firstId);

  console.log("2. Add from messages");
  const r3 = await memory.add(
    [
      { role: "user", content: "I work on NLP and like Rust." },
      { role: "assistant", content: "Noted: NLP and Rust." },
    ],
    userId,
  );
  console.log("   Created id:", r3.results[0].id);

  console.log("3. Search: 'What does Alice prefer?'");
  const search = await memory.search("What does Alice prefer?", userId, 5);
  search.results.forEach((m, i) =>
    console.log(`   [${i + 1}] score=${m.score} ${JSON.stringify(m.content.slice(0, 50))}`),
  );

  console.log("4. Get by id");
  const one = await memory.get(firstId, userId);
  console.log("   ", one?.content);

  console.log("5. Delete");
  console.log("   deleted:", await memory.delete(firstId, userId));

  console.log("Done.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});

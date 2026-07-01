#!/usr/bin/env python3
"""
mem1 Python SDK example (async) — run against a local mem1-server.

Prerequisites:
  Start mem1-server:  cd mem1-server && cargo run

Run (no pip install needed):
  cd python && python examples/example.py
"""
import asyncio
from pathlib import Path
import sys

# Allow importing mem1 from src when not installed
if __name__ == "__main__":
    _root = Path(__file__).resolve().parent.parent
    _src = _root / "src"
    if _src.is_dir() and str(_src) not in sys.path:
        sys.path.insert(0, str(_src))

from mem1 import Memory


async def main() -> None:
    base_url = "http://127.0.0.1:8080"
    user_id = "alice"

    print("mem1 SDK example (async) (base_url=%r, user_id=%r)\n" % (base_url, user_id))

    async with Memory(base_url=base_url) as memory:
        print("1. Add text memory")
        r = await memory.add("Alice prefers dark mode and uses Python for ML.", user_id=user_id)
        print("   Response:", r)
        first_id = r["results"][0]["id"]
        print("   Created id:", first_id)

        print("\n2. Add another text memory")
        r2 = await memory.add("Alice's favorite editor is VS Code.", user_id=user_id)
        print("   Created id:", r2["results"][0]["id"])

        print("\n3. Add from messages (conversation)")
        messages = [
            {"role": "user", "content": "I work on NLP and like Rust."},
            {"role": "assistant", "content": "Noted: NLP and Rust."},
        ]
        r3 = await memory.add(messages, user_id=user_id)
        print("   Created id:", r3["results"][0]["id"])

        print("\n4. Search: 'What does Alice prefer?'")
        search = await memory.search("What does Alice prefer?", user_id=user_id, limit=5)
        for i, m in enumerate(search["results"], 1):
            print("   [%d] id=%s score=%s content=%r" % (i, m["id"], m.get("score"), m["content"][:50]))

        print("\n5. Search: 'editor'")
        search2 = await memory.search("editor", user_id=user_id, limit=5)
        for i, m in enumerate(search2["results"], 1):
            print("   [%d] %r" % (i, m["content"]))

        print("\n6. Get memory by id")
        one = await memory.get(first_id, user_id=user_id)
        print("   %r" % (one and one["content"]))

        print("\n7. Delete first memory")
        ok = await memory.delete(first_id, user_id=user_id)
        print("   deleted:", ok)

        print("\n8. Get again (should be missing)")
        print("   get result:", await memory.get(first_id, user_id=user_id))

        print("\n9. Search again")
        search3 = await memory.search("Alice", user_id=user_id, limit=5)
        print("   results count:", len(search3["results"]))

    print("\nDone.")


if __name__ == "__main__":
    asyncio.run(main())

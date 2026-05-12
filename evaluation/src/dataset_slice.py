import argparse
import copy
import json
import re
from pathlib import Path


EVIDENCE_DAY_RE = re.compile(r"\bD(\d+)\s*:")
SESSION_KEY_RE = re.compile(r"^session_(\d+)(?:_date_time)?$")


def _evidence_session_numbers(item: dict) -> set[str]:
    days: set[str] = set()
    for qa in item.get("qa", []):
        for evidence in qa.get("evidence", []) or []:
            if not isinstance(evidence, str):
                continue
            match = EVIDENCE_DAY_RE.search(evidence)
            if match:
                days.add(match.group(1))
    return days


def _keep_only_evidence_sessions(item: dict) -> None:
    days = _evidence_session_numbers(item)
    conversation = item.get("conversation")
    if not isinstance(conversation, dict) or not days:
        return

    filtered = {}
    for key, value in conversation.items():
        if key in ("speaker_a", "speaker_b"):
            filtered[key] = value
            continue
        match = SESSION_KEY_RE.match(key)
        if match and match.group(1) in days:
            filtered[key] = value
    item["conversation"] = filtered


def slice_locomo_dataset(
    source_path: str | Path,
    output_path: str | Path,
    conversations: int,
    qas_per_conversation: int,
    evidence_only: bool = False,
) -> int:
    source_path = Path(source_path)
    output_path = Path(output_path)
    with source_path.open("r") as f:
        data = json.load(f)

    selected = copy.deepcopy(data[: max(conversations, 0)])
    if qas_per_conversation > 0:
        for item in selected:
            if isinstance(item.get("qa"), list):
                item["qa"] = item["qa"][:qas_per_conversation]
    if evidence_only:
        for item in selected:
            _keep_only_evidence_sessions(item)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w") as f:
        json.dump(selected, f, indent=2)
    return len(selected)


def main() -> None:
    parser = argparse.ArgumentParser(description="Create a small LOCOMO-format dataset slice")
    parser.add_argument("--source", default="dataset/locomo10.json")
    parser.add_argument("--output", default="dataset/smoke_locomo.json")
    parser.add_argument("--conversations", type=int, default=1)
    parser.add_argument("--qas-per-conversation", type=int, default=8)
    parser.add_argument("--evidence-only", action="store_true")
    args = parser.parse_args()

    written = slice_locomo_dataset(
        args.source,
        args.output,
        args.conversations,
        args.qas_per_conversation,
        evidence_only=args.evidence_only,
    )
    print(f"{args.output} ready ({written} conversations)")


if __name__ == "__main__":
    main()

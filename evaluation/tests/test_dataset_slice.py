import json

from src.dataset_slice import slice_locomo_dataset


def test_slice_locomo_dataset_limits_conversations_and_qas(tmp_path):
    source = tmp_path / "source.json"
    output = tmp_path / "smoke.json"
    source.write_text(
        json.dumps(
            [
                {
                    "conversation": {"speaker_a": "Alice", "speaker_b": "Bob"},
                    "qa": [{"question": "q1"}, {"question": "q2"}, {"question": "q3"}],
                },
                {
                    "conversation": {"speaker_a": "Cody", "speaker_b": "Dana"},
                    "qa": [{"question": "q4"}],
                },
            ]
        )
    )

    written = slice_locomo_dataset(source, output, conversations=1, qas_per_conversation=2)

    assert written == 1
    sliced = json.loads(output.read_text())
    assert len(sliced) == 1
    assert [qa["question"] for qa in sliced[0]["qa"]] == ["q1", "q2"]


def test_slice_locomo_dataset_keeps_all_qas_when_limit_is_zero(tmp_path):
    source = tmp_path / "source.json"
    output = tmp_path / "smoke.json"
    source.write_text(
        json.dumps(
            [
                {
                    "conversation": {"speaker_a": "Alice", "speaker_b": "Bob"},
                    "qa": [{"question": "q1"}, {"question": "q2"}],
                }
            ]
        )
    )

    slice_locomo_dataset(source, output, conversations=1, qas_per_conversation=0)

    sliced = json.loads(output.read_text())
    assert [qa["question"] for qa in sliced[0]["qa"]] == ["q1", "q2"]


def test_slice_locomo_dataset_can_keep_only_evidence_sessions(tmp_path):
    source = tmp_path / "source.json"
    output = tmp_path / "smoke.json"
    source.write_text(
        json.dumps(
            [
                {
                    "conversation": {
                        "speaker_a": "Alice",
                        "speaker_b": "Bob",
                        "session_1_date_time": "2023-01-01",
                        "session_1": [{"speaker": "Alice", "text": "used by q1"}],
                        "session_2_date_time": "2023-01-02",
                        "session_2": [{"speaker": "Bob", "text": "not selected"}],
                        "session_3_date_time": "2023-01-03",
                        "session_3": [{"speaker": "Alice", "text": "used by q2"}],
                    },
                    "qa": [
                        {"question": "q1", "evidence": ["D1:1"]},
                        {"question": "q2", "evidence": ["D3:1"]},
                        {"question": "q3", "evidence": ["D2:1"]},
                    ],
                }
            ]
        )
    )

    slice_locomo_dataset(
        source,
        output,
        conversations=1,
        qas_per_conversation=2,
        evidence_only=True,
    )

    conversation = json.loads(output.read_text())[0]["conversation"]
    assert set(conversation) == {
        "speaker_a",
        "speaker_b",
        "session_1_date_time",
        "session_1",
        "session_3_date_time",
        "session_3",
    }

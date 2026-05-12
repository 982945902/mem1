from src.mem1_add import build_speaker_messages


def test_build_speaker_messages_attaches_session_date_as_valid_at():
    conversation = {
        "speaker_a": "Caroline",
        "speaker_b": "Melanie",
        "session_1_date_time": "1:56 pm on 8 May, 2023",
        "session_1": [
            {"speaker": "Caroline", "text": "I went to a support group yesterday."},
            {"speaker": "Melanie", "text": "I painted that lake sunrise last year."},
        ],
    }

    speaker_a, speaker_b, messages_a, messages_b = build_speaker_messages(conversation)

    assert (speaker_a, speaker_b) == ("Caroline", "Melanie")
    assert messages_a == [
        {
            "role": "user",
            "content": "Caroline: I went to a support group yesterday.",
            "metadata": {"valid_at": "1:56 pm on 8 May, 2023"},
        }
    ]
    assert messages_b == [
        {
            "role": "user",
            "content": "Melanie: I painted that lake sunrise last year.",
            "metadata": {"valid_at": "1:56 pm on 8 May, 2023"},
        }
    ]

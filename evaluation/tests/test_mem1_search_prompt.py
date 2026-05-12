from src.mem1_search import build_answer_prompt


def test_answer_prompt_tells_model_to_resolve_relative_dates():
    prompt = build_answer_prompt(
        speaker_1_user_id="Caroline",
        speaker_2_user_id="Melanie",
        speaker_1_memories="Caroline: I went yesterday. (Date: 8 May 2023)",
        speaker_2_memories="Melanie: I painted it last year. (Date: 8 May 2023)",
        question="When did Melanie paint a sunrise?",
    )

    assert "Resolve relative dates" in prompt
    assert "last year" in prompt
    assert "memory Date" in prompt

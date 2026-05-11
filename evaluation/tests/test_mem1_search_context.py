from src.mem1_search import memory_context_from_response


def test_memory_context_prefers_formatted_context_over_raw_results():
    resp = {
        "formatted_context": "<FACTS>\nAlice keeps her passport in a blue pouch.\n</FACTS>",
        "results": [{"content": "raw content should not be used"}],
    }

    assert memory_context_from_response(resp) == resp["formatted_context"]


def test_memory_context_falls_back_to_result_contents():
    resp = {
        "results": [
            {"content": "Alice likes tea."},
            {"content": "Alice visits Paris often."},
        ]
    }

    assert memory_context_from_response(resp) == "Alice likes tea.\nAlice visits Paris often."

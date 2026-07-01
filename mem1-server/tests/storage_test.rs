//! Unit tests for storage layer (T011). Run with: cargo test --test storage_test

use mem1_server::memory::model::{Memory, Session};
use mem1_server::storage::{self, MemoryFilters, MemoryStore, SessionStore};
use std::collections::HashMap;

#[test]
fn memory_new_sets_id_and_timestamps() {
    let m = Memory::new("hello".to_string(), "user1".to_string(), HashMap::new());
    assert!(!m.id.is_empty());
    assert_eq!(m.content, "hello");
    assert_eq!(m.user_id, "user1");
    assert!(!m.created_at.is_empty());
    assert_eq!(m.created_at, m.updated_at);
}

async fn test_store(name: &str) -> (String, storage::SurrealMemoryStore) {
    let db_path = std::env::temp_dir().join(format!(
        "mem1-storage-test-{}-{}",
        name,
        uuid::Uuid::new_v4()
    ));
    let db_path = db_path.to_string_lossy().to_string();
    let db = storage::connect(&db_path).await.unwrap();
    storage::ensure_schema(&db).await.unwrap();
    (db_path, storage::store(db))
}

#[tokio::test]
async fn update_changes_content_metadata_and_records_history() {
    let (db_path, store) = test_store("update-history").await;
    let updated_embedding = vec![0.3; 384];
    let created = store
        .add(&Memory::new(
            "old content".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("scope".to_string(), serde_json::json!("project"));
    let updated = store
        .update(
            &created.id,
            "u1",
            Some("new content".to_string()),
            Some(updated_embedding.clone()),
            Some(metadata),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.content, "new content");
    assert_eq!(updated.embedding, Some(updated_embedding));
    assert_eq!(
        updated.metadata.get("scope").and_then(|v| v.as_str()),
        Some("project")
    );
    assert!(updated.updated_at >= updated.created_at);

    let history = store.history(&created.id, "u1").await.unwrap();
    let operations: Vec<_> = history.iter().map(|h| h.operation.as_str()).collect();
    assert_eq!(operations, vec!["ADD", "UPDATE"]);

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn delete_all_is_scoped_to_user_and_filters() {
    let (db_path, store) = test_store("delete-all").await;
    let mut project_meta = HashMap::new();
    project_meta.insert("scope".to_string(), serde_json::json!("project"));
    let mut session_meta = HashMap::new();
    session_meta.insert("scope".to_string(), serde_json::json!("session"));

    store
        .add(&Memory::new(
            "project memory".to_string(),
            "u1".to_string(),
            project_meta,
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "session memory".to_string(),
            "u1".to_string(),
            session_meta,
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "other user memory".to_string(),
            "u2".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let mut filters = MemoryFilters::default();
    filters
        .metadata
        .insert("scope".to_string(), "project".to_string());
    let deleted = store.delete_all("u1", &filters).await.unwrap();

    assert_eq!(deleted, 1);
    assert_eq!(
        store
            .list_by_user("u1", 10, 0, &MemoryFilters::default())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .list_by_user("u2", 10, 0, &MemoryFilters::default())
            .await
            .unwrap()
            .len(),
        1
    );

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn users_and_reset_cover_all_memories() {
    let (db_path, store) = test_store("users-reset").await;
    store
        .add(&Memory::new(
            "one".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "two".to_string(),
            "u2".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let mut users = store.list_users().await.unwrap();
    users.sort();
    assert_eq!(users, vec!["u1", "u2"]);

    let deleted = store.reset().await.unwrap();
    assert_eq!(deleted, 2);
    assert!(store.list_users().await.unwrap().is_empty());

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn search_expands_memories_connected_by_graph_entities() {
    let (db_path, store) = test_store("graph-search").await;
    let passport = store
        .add(&Memory::new(
            "Alice misplaced her passport at the airport".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    let hotel = store
        .add(&Memory::new(
            "Alice booked a hotel near the museum".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "Bob prefers train travel".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let rows = store
        .search("u1", "passport", None, 5, &MemoryFilters::default())
        .await
        .unwrap();
    let ids: Vec<_> = rows.into_iter().map(|(m, _)| m.id).collect();

    assert!(ids.contains(&passport.id));
    assert!(ids.contains(&hotel.id));

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn graph_search_matches_lowercase_query_entities() {
    let (db_path, store) = test_store("graph-lowercase-query").await;
    let caroline_fact = store
        .add(&Memory::new(
            "Caroline keeps her travel documents in a blue pouch".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "Bob stores receipts in a kitchen drawer".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let rows = store
        .search(
            "u1",
            "where are caroline papers stored",
            None,
            5,
            &MemoryFilters::default(),
        )
        .await
        .unwrap();
    let ids: Vec<_> = rows.into_iter().map(|(m, _)| m.id).collect();

    assert!(ids.contains(&caroline_fact.id));

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn graph_search_indexes_acronym_entities() {
    let (db_path, store) = test_store("graph-acronym-query").await;
    let support_group = store
        .add(&Memory::new(
            "Caroline attended an LGBTQ support group on Sunday".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    store
        .add(&Memory::new(
            "Caroline bought coffee before work".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let rows = store
        .search(
            "u1",
            "which lgbtq event did she attend",
            None,
            5,
            &MemoryFilters::default(),
        )
        .await
        .unwrap();
    let ids: Vec<_> = rows.into_iter().map(|(m, _)| m.id).collect();

    assert!(ids.contains(&support_group.id));

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn graph_search_does_not_expand_only_by_speaker_prefix() {
    let (db_path, store) = test_store("graph-speaker-prefix").await;
    let passport = store
        .add(&Memory::new(
            "Alice: misplaced her passport at the airport".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();
    let hotel = store
        .add(&Memory::new(
            "Alice: booked a hotel near the museum".to_string(),
            "u1".to_string(),
            HashMap::new(),
        ))
        .await
        .unwrap();

    let rows = store
        .search("u1", "passport", None, 5, &MemoryFilters::default())
        .await
        .unwrap();
    let ids: Vec<_> = rows.into_iter().map(|(m, _)| m.id).collect();

    assert!(ids.contains(&passport.id));
    assert!(!ids.contains(&hotel.id));

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn session_create_get_list_and_user_isolation() {
    let (db_path, store) = test_store("session-crud").await;

    let s = Session::new(
        "s1".to_string(),
        "u1".to_string(),
        Some("chat".to_string()),
        HashMap::new(),
    );
    let created = store.create_session(&s).await.unwrap();
    assert_eq!(created.id, "s1");
    assert_eq!(created.name.as_deref(), Some("chat"));

    // get scoped to owner
    assert!(store.get_session("s1", "u1").await.unwrap().is_some());
    assert!(store.get_session("s1", "other").await.unwrap().is_none());

    // another user's session isn't listed
    let s2 = Session::new("s2".to_string(), "u2".to_string(), None, HashMap::new());
    store.create_session(&s2).await.unwrap();
    let listed = store.list_sessions("u1").await.unwrap();
    assert_eq!(
        listed.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(),
        vec!["s1"]
    );

    let _ = std::fs::remove_dir_all(db_path);
}

#[tokio::test]
async fn delete_session_keeps_or_cascades_memories() {
    let (db_path, store) = test_store("session-cascade").await;

    // A memory tagged with run_id=s1.
    let mut md = HashMap::new();
    md.insert("run_id".to_string(), serde_json::json!("s1"));
    store
        .add(&Memory::new(
            "Alice likes tea".to_string(),
            "u1".to_string(),
            md,
        ))
        .await
        .unwrap();
    store
        .create_session(&Session::new(
            "s1".to_string(),
            "u1".to_string(),
            None,
            HashMap::new(),
        ))
        .await
        .unwrap();

    // delete_session removes only the session row; the memory survives.
    assert!(store.delete_session("s1", "u1").await.unwrap());
    assert!(store.get_session("s1", "u1").await.unwrap().is_none());
    let mut run_filter = MemoryFilters::default();
    run_filter
        .metadata
        .insert("run_id".to_string(), "s1".to_string());
    let kept = store.list_by_user("u1", 10, 0, &run_filter).await.unwrap();
    assert_eq!(
        kept.len(),
        1,
        "memory should survive non-cascade session delete"
    );

    // Cascade: caller deletes the session's memories via delete_all + run_id filter.
    let deleted = store.delete_all("u1", &run_filter).await.unwrap();
    assert_eq!(deleted, 1);
    let after = store.list_by_user("u1", 10, 0, &run_filter).await.unwrap();
    assert!(after.is_empty());

    let _ = std::fs::remove_dir_all(db_path);
}

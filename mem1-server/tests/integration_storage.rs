//! Integration tests for SurrealDB-backed storage (T012).
//! Run manually with: cargo test --test integration_storage -- --ignored

use mem1_server::memory::model::Memory;
use mem1_server::storage::{connect, ensure_schema, store};
use mem1_server::MemoryStore;
use std::collections::HashMap;

fn test_db_path(name: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock ok")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("mem1_test_{name}_{nanos}.db"));
    dir.to_string_lossy().to_string()
}

#[tokio::test]
#[ignore]
async fn storage_add_get_search_flow() {
    let db = connect(&test_db_path("add_get_search")).await.expect("connect");
    ensure_schema(&db).await.expect("ensure schema");
    let store = store(db);

    let user_id = "u1".to_string();
    let mem = Memory::new("Alice likes tea".to_string(), user_id.clone(), HashMap::new());
    let created = store.add(&mem).await.expect("add");

    let got = store
        .get(&created.id, &user_id)
        .await
        .expect("get")
        .expect("exists");
    assert_eq!(got.content, "Alice likes tea");

    let results = store
        .search(&user_id, "Alice tea", None, 10)
        .await
        .expect("search");
    assert!(!results.is_empty());
}

#[tokio::test]
#[ignore]
async fn storage_delete_all_by_user_only_affects_target_user() {
    let db = connect(&test_db_path("delete_all")).await.expect("connect");
    ensure_schema(&db).await.expect("ensure schema");
    let store = store(db);

    let user_a = "user_a".to_string();
    let user_b = "user_b".to_string();

    let a1 = Memory::new("alpha one".to_string(), user_a.clone(), HashMap::new());
    let a2 = Memory::new("alpha two".to_string(), user_a.clone(), HashMap::new());
    let b1 = Memory::new("beta one".to_string(), user_b.clone(), HashMap::new());

    store.add(&a1).await.expect("add a1");
    store.add(&a2).await.expect("add a2");
    store.add(&b1).await.expect("add b1");

    let deleted = store
        .delete_all_by_user(&user_a)
        .await
        .expect("delete all by user");
    assert_eq!(deleted, 2);

    let a_results = store.search(&user_a, "alpha", None, 10).await.expect("search a");
    let b_results = store.search(&user_b, "beta", None, 10).await.expect("search b");

    assert!(a_results.is_empty());
    assert!(!b_results.is_empty());
}

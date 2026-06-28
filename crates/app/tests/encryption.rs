use db::pool::connect_with_key;

#[tokio::test]
async fn encrypted_roundtrip_and_wrong_key_fails() {
    let path = "/tmp/hrk_enc_test.db";
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path));
    let _ = std::fs::remove_file(format!("{}-shm", path));
    // connect_with_key sets create_if_missing(true); plain file url, no ?mode=rwc.
    let url = format!("sqlite:{}", path);

    // create encrypted DB, migrate, write a row
    {
        let pool = connect_with_key(&url, Some("correct-horse-battery-staple")).await.unwrap();
        sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO skills (name) VALUES ('Rust')").execute(&pool).await.unwrap();
    }

    // reopen with the CORRECT key -> reads the row
    {
        let pool = connect_with_key(&url, Some("correct-horse-battery-staple")).await.unwrap();
        let n: (i64,) = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await.unwrap();
        assert_eq!(n.0, 1);
    }

    // reopen with a WRONG key -> under SQLCipher + sqlx, the non-key PRAGMAs run after
    // `PRAGMA key` and attempt to read the encrypted header, so the connection itself
    // fails (code 26 "file is not a database"). Either connect or the first query must fail.
    {
        let res = connect_with_key(&url, Some("wrong-key")).await;
        let failed = match res {
            Ok(pool) => {
                let q: Result<(i64,), _> =
                    sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await;
                q.is_err()
            }
            Err(_) => true,
        };
        assert!(failed, "wrong passphrase must fail to connect or decrypt/read");
    }

    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path));
    let _ = std::fs::remove_file(format!("{}-shm", path));
}
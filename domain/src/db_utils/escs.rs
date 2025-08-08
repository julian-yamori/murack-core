/// 文字列値を SQL で使用する値にエスケープ
pub fn escs(v: &str) -> String {
    //'で囲む 更にSQLインジェクション等対策
    format!("'{}'", v.replace('\'', "''"))
}

#[cfg(test)]
mod test_escs {
    use sqlx::PgPool;

    use super::escs;

    async fn compare_escs_result(pool: &PgPool, to_escaped: &str) -> anyhow::Result<()> {
        let sql = format!("SELECT {}", escs(to_escaped));
        let actual: String = sqlx::query_scalar(&sql).fetch_one(pool).await?;

        assert_eq!(&actual, to_escaped);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_normal_str(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "hello").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_empty(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_japanese(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "日本語").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_emoji(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "✔️🦀").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_line_breaks(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "first\nsecond\r\nthird\rfourth").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_one_quoat(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "'").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_two_quoats(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "''").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_three_quoats(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "'''").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_four_quoats(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "''''").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_control_chars(pool: PgPool) -> anyhow::Result<()> {
        // 0x01 から 0x19 までの制御文字で確認

        // 0x00 については、PostgreSQL 側で NULL 文字が検出されると失敗する。
        // NULL byte injection 対策のためその動作を期待値とし、assert_failure_by_null_byte_injection() で確認する

        let control_chars: String = ('\x01'..'\x20').collect();

        compare_escs_result(&pool, &control_chars).await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_injection(pool: PgPool) -> anyhow::Result<()> {
        compare_escs_result(&pool, "''; SELECT password FROM users --").await
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn assert_failure_by_null_byte_injection(pool: PgPool) -> anyhow::Result<()> {
        let escaped = escs("''\0");

        let sql = format!("SELECT {escaped}");
        let result: sqlx::Result<String> = sqlx::query_scalar(&sql).fetch_one(&pool).await;

        assert!(result.is_err(), "NULL injected SQL is succeeded");

        Ok(())
    }
}

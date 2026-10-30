/*
level：INFO、WARN、ERROR 等級。
target：產生日誌的 Rust module 路徑。
message：日誌訊息。
*/
CREATE TABLE ddns_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT, 
    level TEXT NOT NULL,
    target TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_dns_logs_created_at ON ddns_logs (created_at);
-- OAuth 2.0 Server Database Schema
-- SQLite Schema for login-server

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_users_email ON users(email);

-- OAuth Clients table
CREATE TABLE IF NOT EXISTS oauth_clients (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    client_type TEXT NOT NULL CHECK(client_type IN ('public', 'confidential')),
    client_secret TEXT,
    redirect_uris TEXT NOT NULL, -- JSON array
    allowed_scopes TEXT NOT NULL, -- Space-separated scopes
    created_at TEXT NOT NULL
);

CREATE INDEX idx_oauth_clients_type ON oauth_clients(client_type);

-- Authorization Codes table
CREATE TABLE IF NOT EXISTS authorization_codes (
    code TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    redirect_uri TEXT NOT NULL,
    scope TEXT NOT NULL,
    code_challenge TEXT,
    code_challenge_method TEXT,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_auth_codes_expires_at ON authorization_codes(expires_at);
CREATE INDEX idx_auth_codes_used ON authorization_codes(used);

-- Access Tokens table
CREATE TABLE IF NOT EXISTS access_tokens (
    token TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_access_tokens_user_id ON access_tokens(user_id);
CREATE INDEX idx_access_tokens_expires_at ON access_tokens(expires_at);

-- Refresh Tokens table
CREATE TABLE IF NOT EXISTS refresh_tokens (
    token TEXT PRIMARY KEY NOT NULL,
    access_token_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (client_id) REFERENCES oauth_clients(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_client_id ON refresh_tokens(client_id);
CREATE INDEX idx_refresh_tokens_access_token_id ON refresh_tokens(access_token_id);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens(revoked);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);

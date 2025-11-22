# Docker Deployment Guide

This guide explains how to deploy the OAuth 2.0 Server using Docker and Docker Compose.

## Prerequisites

- Docker 20.10+ installed
- Docker Compose 2.0+ installed
- At least 2GB of available disk space

## Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd login-server
```

### 2. Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` and set your configuration:

```env
# REQUIRED: JWT Secret (minimum 32 characters)
JWT_SECRET=your-production-secret-key-change-this-min-32-chars

# Database (SQLite)
DATABASE_URL=sqlite:/app/data/oauth.db

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# CORS Configuration
CORS_ALLOWED_ORIGINS=http://localhost:3000,https://yourdomain.com

# Token Expiration (in seconds)
TOKEN_ACCESS_TOKEN_EXPIRATION_SECONDS=3600
TOKEN_REFRESH_TOKEN_EXPIRATION_SECONDS=2592000
TOKEN_AUTHORIZATION_CODE_EXPIRATION_SECONDS=600

# JWT Configuration
JWT_EXPIRATION_SECONDS=3600
JWT_ISSUER=oauth-server

# Logging
RUST_LOG=info
```

### 3. Build and Run

```bash
# Build the Docker image
docker-compose build

# Start the server
docker-compose up -d

# View logs
docker-compose logs -f oauth-server
```

### 4. Verify Deployment

The server should now be running on `http://localhost:8080` (or your configured port).

Check the health status:

```bash
docker-compose ps
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | Yes | - | JWT signing secret (min 32 chars) |
| `DATABASE_URL` | No | `sqlite:/app/data/oauth.db` | SQLite database path |
| `SERVER_HOST` | No | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | No | `8080` | Server port |
| `CORS_ALLOWED_ORIGINS` | No | `http://localhost:3000` | Comma-separated CORS origins |
| `TOKEN_ACCESS_TOKEN_EXPIRATION_SECONDS` | No | `3600` | Access token lifetime (1 hour) |
| `TOKEN_REFRESH_TOKEN_EXPIRATION_SECONDS` | No | `2592000` | Refresh token lifetime (30 days) |
| `TOKEN_AUTHORIZATION_CODE_EXPIRATION_SECONDS` | No | `600` | Auth code lifetime (10 minutes) |
| `JWT_ISSUER` | No | `oauth-server` | JWT issuer claim |
| `RUST_LOG` | No | `info` | Log level (error/warn/info/debug/trace) |

### Data Persistence

The SQLite database is stored in a Docker volume (`oauth-data`) to persist data across container restarts.

To back up the database:

```bash
# Create a backup
docker-compose exec oauth-server sqlite3 /app/data/oauth.db ".backup '/app/data/backup.db'"

# Copy backup to host
docker cp oauth-server:/app/data/backup.db ./backup.db
```

## Production Deployment

### Security Recommendations

1. **Generate a Strong JWT Secret:**
   ```bash
   openssl rand -base64 48
   ```

2. **Use HTTPS:** Deploy behind a reverse proxy (nginx, Traefik) with SSL/TLS

3. **Configure CORS:** Only allow trusted origins

4. **Regular Backups:** Schedule automated database backups

5. **Monitor Logs:** Set up log aggregation (e.g., ELK stack, Grafana Loki)

### Reverse Proxy Example (nginx)

```nginx
server {
    listen 443 ssl http2;
    server_name oauth.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Management Commands

### View Logs

```bash
# Follow logs
docker-compose logs -f oauth-server

# View last 100 lines
docker-compose logs --tail=100 oauth-server
```

### Restart Server

```bash
docker-compose restart oauth-server
```

### Stop Server

```bash
docker-compose down
```

### Rebuild After Code Changes

```bash
docker-compose down
docker-compose build --no-cache
docker-compose up -d
```

### Access Shell in Container

```bash
docker-compose exec oauth-server /bin/sh
```

### Remove All Data (CAUTION)

```bash
# This will delete the database!
docker-compose down -v
```

## Troubleshooting

### Server Won't Start

1. Check logs:
   ```bash
   docker-compose logs oauth-server
   ```

2. Verify environment variables:
   ```bash
   docker-compose config
   ```

3. Ensure JWT_SECRET is set and > 32 characters

### Database Errors

1. Check database permissions:
   ```bash
   docker-compose exec oauth-server ls -la /app/data
   ```

2. Reset database (CAUTION - deletes all data):
   ```bash
   docker-compose down
   docker volume rm login-server_oauth-data
   docker-compose up -d
   ```

### Port Already in Use

Change the port in `.env`:

```env
SERVER_PORT=8081
```

Then restart:

```bash
docker-compose down
docker-compose up -d
```

## Performance Tuning

### Resource Limits

Add resource limits to `docker-compose.yml`:

```yaml
services:
  oauth-server:
    # ... existing configuration ...
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 512M
        reservations:
          memory: 256M
```

### Database Optimization

For better SQLite performance, add to `.env`:

```env
DATABASE_URL=sqlite:/app/data/oauth.db?mode=rwc&cache=shared&_journal_mode=WAL
```

## API Endpoints

Once deployed, the following endpoints are available:

### User Management
- `POST /users/register` - Register new user
- `POST /auth/login` - Authenticate user
- `GET /users/{id}` - Get user info
- `POST /users/{id}/password` - Change password

### OAuth Client Management
- `POST /clients/register` - Register OAuth client
- `GET /clients/{id}` - Get client info
- `DELETE /clients/{id}` - Delete client

### OAuth Flow
- `GET /authorize` - Authorization endpoint
- `POST /authorize/consent` - User consent
- `POST /token` - Token exchange
- `POST /revoke` - Revoke token
- `POST /introspect` - Introspect token

## Support

For issues or questions:
- Check logs: `docker-compose logs -f oauth-server`
- Review configuration: `docker-compose config`
- Verify environment variables are set correctly
- Ensure JWT_SECRET meets minimum length requirement

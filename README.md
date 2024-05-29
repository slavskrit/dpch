# Telegram bot for downloading purposes.

## Useful commands

### Building for server:
```
docker build --platform linux/amd64 .
```

```
docker tag c9ba92421a7b dimasikpupsik/dpch
```

```
docker push dimasikpupsik/dpch
```

### Local run

```
docker-compose --env-file .env up --build -w
```

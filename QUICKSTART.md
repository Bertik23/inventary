# Quick Start Guide

## Prerequisites

1. Install Rust: https://rustup.rs/
2. Install Trunk: `cargo install trunk`
3. Install Diesel CLI: `cargo install diesel_cli --no-default-features --features sqlite`

## Backend Setup (5 minutes)

```bash
cd backend
cp .env.example .env
# Edit .env if needed (default SQLite setup should work)
diesel migration run
cargo run
```

The backend will start on `http://127.0.0.1:8080`

## Frontend Setup (5 minutes)

```bash
cd frontend
trunk serve
```

The frontend will be available at `http://127.0.0.1:8081`

## First Use

1. Open `http://127.0.0.1:8081` in your browser
2. Click "Add Item"
3. Search for a product (e.g., "coca cola")
4. Select a product and add it to inventory
5. Click "Show Inventory" to see your items

## Troubleshooting

### Backend won't start
- Make sure port 8080 is not in use
- Check that `DATABASE_URL` in `.env` is correct
- Ensure migrations ran successfully

### Frontend won't build
- Make sure `trunk` is installed: `cargo install trunk`
- Check that you're in the `frontend` directory
- Try `trunk clean` and rebuild

### Database errors
- For SQLite: Make sure the database file path is writable
- Run migrations: `diesel migration run`
- Check `.env` file configuration

### CORS errors
- Make sure backend is running on port 8080
- Backend has CORS enabled for development
- Check browser console for specific errors

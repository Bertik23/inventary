# Inventory Management System

A simple inventory management system built with Rust, featuring both a backend API and a Progressive Web App (PWA) frontend.

## Features

- **Add Items**: Scan barcodes or manually search for products
- **Remove Items**: Remove items from inventory by scanning or searching
- **View Inventory**: Display all items in your inventory with quantities
- **OpenFoodFacts Integration**: Automatically fetch product information from OpenFoodFacts.org
- **Database Support**: Choose between SQLite (default) or PostgreSQL
- **PWA**: Installable Progressive Web App that works offline

## Prerequisites

- Rust (latest stable version)
- For PostgreSQL: PostgreSQL server installed and running
- For building frontend: `trunk` (install with `cargo install trunk`)

## Setup

### Backend

1. Navigate to the backend directory:
```bash
cd backend
```

2. Copy the example environment file:
```bash
cp .env.example .env
```

3. Edit `.env` to configure your database and OpenFoodFacts API:
   - For SQLite (default): `DATABASE_URL=inventory.db` and `DATABASE_TYPE=sqlite`
   - For PostgreSQL: `DATABASE_URL=postgres://user:password@localhost/inventory` and `DATABASE_TYPE=postgres`
   - Optional: `OFF_USER_AGENT=Inventary/0.1.0 (your-email@example.com)` - Custom User-Agent for OpenFoodFacts API (required format: AppName/Version (ContactEmail))

4. Install Diesel CLI (if not already installed):
```bash
cargo install diesel_cli --no-default-features --features sqlite
# Or for PostgreSQL:
# cargo install diesel_cli --no-default-features --features postgres
```

5. Run migrations:
```bash
diesel migration run
```

6. Start the backend server:
```bash
cargo run
```

The backend will start on `http://127.0.0.1:8080`

### Frontend

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Build and serve the frontend:
```bash
trunk serve
```

The frontend will be available at `http://127.0.0.1:8081`

## Usage

1. Open the frontend in your browser
2. Use the main menu to:
   - **Add Item**: Scan a barcode or search for a product manually
   - **Remove Item**: Remove items by scanning or searching
   - **Show Inventory**: View all items in your inventory

## Barcode Scanning

The app includes full barcode scanning functionality with camera access:

- **Native BarcodeDetector API**: Automatically used in Chrome/Edge browsers (best performance)
- **ZXing Fallback**: Optional JavaScript library for Firefox/Safari (uncomment script tag in `index.html`)
- **Manual Entry**: Always available as a fallback - users can type barcodes directly

**Browser Support:**
- ✅ **Chrome/Edge**: Full support via BarcodeDetector API (no additional setup needed)
- ✅ **Firefox/Safari**: Requires ZXing library (uncomment the script tag in `frontend/index.html`)
- ✅ **All browsers**: Manual barcode entry always works

**To enable ZXing for Firefox/Safari:**
1. Open `frontend/index.html`
2. Uncomment the ZXing script tag: `<script src="https://unpkg.com/@zxing/library@latest"></script>`
3. Rebuild the frontend

The scanner supports common barcode formats: EAN-13, EAN-8, UPC-A, UPC-E, Code 128, Code 39, and QR codes.

## Database

The system uses Diesel ORM and currently supports SQLite (PostgreSQL support can be added):

- **SQLite**: Simple file-based database, perfect for local use (default and fully supported)
- **PostgreSQL**: Full-featured database for production deployments (requires additional handler implementation)

Currently, SQLite is the default and fully supported database. To use PostgreSQL, you would need to:
1. Create Postgres-specific handlers (similar to `handlers.rs` but using `diesel::PgConnection`)
2. Update `main.rs` to use the Postgres handlers when `DATABASE_TYPE=postgres`

The database schema is compatible with both SQLite and PostgreSQL.

## API Endpoints

- `GET /api/inventory` - Get all inventory items
- `POST /api/inventory/add` - Add an item to inventory
- `POST /api/inventory/remove` - Remove an item from inventory
- `GET /api/search?q=<query>` - Search for products
- `GET /api/product/<barcode>` - Get product by barcode

## OpenFoodFacts API Integration

This application uses the [OpenFoodFacts API v2](https://openfoodfacts.github.io/openfoodfacts-server/api/) to fetch product information. 

**Important Notes:**
- The API has rate limits: 100 requests/minute for product queries, 10 requests/minute for search queries
- A custom User-Agent is required (format: `AppName/Version (ContactEmail)`)
- You can configure the User-Agent via the `OFF_USER_AGENT` environment variable
- For production use, consider filling out the [API usage form](https://openfoodfacts.github.io/openfoodfacts-server/api/) to help OpenFoodFacts understand your usage

**Rate Limiting:**
- Product lookups: 100 requests per minute
- Search queries: 10 requests per minute
- If limits are exceeded, your IP may be temporarily banned

## Development

### Backend

The backend is built with:
- Actix-web for the HTTP server
- Diesel for database ORM
- OpenFoodFacts API for product data

### Frontend

The frontend is built with:
- Yew for the web framework
- WebAssembly for performance
- PWA capabilities for offline support

## Deployment

### Easy Server Deployment (Docker Combined)

The easiest way to deploy the entire system is using the provided Docker configuration. This builds both the frontend and backend into a single container.

1.  **Clone the repository and navigate to the directory.**
2.  **Run the deployment script:**
    ```bash
    ./deploy_server.sh
    ```
    This script will:
    - Create a `./data` directory for the SQLite database.
    - Build a combined Docker image for the frontend and backend.
    - Start the service on port 8080.

3.  **Access the application:**
    Open `http://your-server-ip:8080` in your browser.

### Manual Server Deployment (Docker Compose)

You can also run things separately using the standard `docker-compose.yml`:
```bash
docker-compose up --build
```

## Mobile App

This application is a **Progressive Web App (PWA)**. You can "install" it on your mobile device without an app store:

1.  **Host the application on a server with HTTPS** (required for PWA).
2.  **On Android (Chrome):** Tap the menu icon and select "Install app" or "Add to Home screen".
3.  **On iOS (Safari):** Tap the "Share" icon and select "Add to Home Screen".

For more details, see `./build_mobile.sh`.

## License

This project is open source and available for use.

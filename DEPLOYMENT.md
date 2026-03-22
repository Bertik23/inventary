# Deployment Guide: Inventary Management System

This guide explains how to deploy the Inventary system on your own server. We've designed it to be as simple as possible, even if you've never deployed a web app before.

## 🚀 Quickest Way: Using Prebuilt Docker Images

If you don't want to build the code yourself, you can use our prebuilt images from the **GitHub Container Registry (GHCR)**. This is the fastest way to get started.

### Option 1: Using Docker Compose (Recommended)

**Create a file named `docker-compose.yml`**:
```yaml
services:
  inventory:
    image: ghcr.io/bertik23/inventary:latest
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
    env_file: .env
    restart: unless-stopped
```

**Run it**:
```bash
docker-compose up -d
```

### Option 2: Using the One-Command Script

If you've downloaded the repository, our deployment script also supports using the prebuilt image:
```bash
chmod +x deploy_server.sh
./deploy_server.sh --prebuilt
```

---

## 🛠️ Configuration (Environment Variables)

The app is configured using "Environment Variables". You can set these in a file named `.env` in the project root.

### Database Settings
| Variable | Description | Default | Example |
| :--- | :--- | :--- | :--- |
| `DATABASE_TYPE` | `sqlite` or `postgres` | `sqlite` | `sqlite` |
| `DATABASE_URL` | Connection string for the DB | (Required) | `sqlite:///app/data/inventory.db` |

### Server Settings
| Variable | Description | Default | Example |
| :--- | :--- | :--- | :--- |
| `HOST` | The IP address to listen on | `0.0.0.0` | `0.0.0.0` |
| `PORT` | The port to listen on | `8080` | `8080` |
| `APP_URL` | The public URL of your app (used for emails) | `http://localhost:8080` | `https://inventory.example.com` |
| `STATIC_FILES_DIR` | Where the frontend files are | (None) | `/app/static` |

### OpenFoodFacts (OFF) Integration
| Variable | Description | Example |
| :--- | :--- | :--- |
| `OFF_USER_AGENT` | Identify your app to OFF | `MyInventory/1.0 (contact@me.com)` |
| `OFF_USERNAME` | Your OpenFoodFacts username | `myuser` |
| `OFF_PASSWORD` | Your OpenFoodFacts password | `mypassword` |

### Email (SMTP) Settings
*Required for password resets and notifications.*

| Variable | Description | Example |
| :--- | :--- | :--- |
| `SMTP_SERVER` | Your SMTP host | `smtp.gmail.com` |
| `SMTP_PORT` | SMTP port (usually 587 or 465) | `587` |
| `SMTP_USER` | SMTP username | `user@gmail.com` |
| `SMTP_PASS` | SMTP password | `app-specific-password` |
| `FROM_EMAIL` | The "From" address for emails | `noreply@example.com` |

---

## 📋 Deployment Examples

### Example 1: Basic Home Server (SQLite)
Perfect for a Raspberry Pi or local server. No complex database setup.

**Create a file named `.env`**:
```env
# Server
PORT=8080
HOST=0.0.0.0
APP_URL=http://192.168.1.50:8080

# Database (Data will be saved in the ./data folder)
DATABASE_TYPE=sqlite
DATABASE_URL=sqlite:///app/data/inventory.db

# Optional: Identify yourself to OpenFoodFacts
OFF_USER_AGENT=HomeInventory/1.0 (me@home.local)
```

**Run it**:
```bash
docker-compose -f docker-compose.prod.yml up -d
```

---

### Example 2: Professional Cloud Deployment (HTTPS + Email)
Use this if you are deploying to a VPS (like DigitalOcean, AWS, or Hetzner) and want a real domain name.

**`.env` file**:
```env
# Server & Public URL
APP_URL=https://inventory.yourdomain.com
PORT=8080

# Database
DATABASE_TYPE=sqlite
DATABASE_URL=sqlite:///app/data/inventory.db

# Email (using Gmail as an example)
SMTP_SERVER=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASS=your-app-password
FROM_EMAIL=Inventory System <your-email@gmail.com>

# OpenFoodFacts
OFF_USER_AGENT=InventoryApp/1.0 (admin@yourdomain.com)
```

**Run it**:
```bash
docker-compose -f docker-compose.prod.yml up -d
```

*Note: You would typically put a Reverse Proxy like **Nginx** or **Caddy** in front of this to handle HTTPS.*

---

## 📱 Mobile App (PWA)

Once your server is running with **HTTPS**, you can "install" the app on your phone:

1.  **Android**: Open the URL in Chrome -> Tap 3 dots -> **Install App**.
2.  **iPhone**: Open the URL in Safari -> Tap Share -> **Add to Home Screen**.

---

## 🧹 Maintenance

### Updating to the latest version
```bash
git pull
./deploy_server.sh
```

### Checking the logs
If something isn't working, check the logs:
```bash
docker-compose -f docker-compose.prod.yml logs -f
```

### Backing up your data
All your inventory data is stored in the `./data` folder. Simply copy this folder to a safe place to back it up.

# Hackernews Sauron

A pure client-side Hacker News clone built with [Sauron](https://github.com/ivanceras/sauron) (Rust WASM framework).

## Features
- ✅ **100% Client-side** - No backend required, pure static deployment
- ✅ **Hash-based routing** - URLs like `#top`, `#item/123`, `#user/pg`
- ✅ **Direct HN API** - Calls HackerNews Firebase API directly (CORS-enabled)
- ✅ **HTML content parsing** - Properly renders HTML entities and tags in comments
- ✅ **Modern Rust WASM** - Built with the latest Sauron framework

## Quick Start

### Prerequisites
```sh
cargo install wasm-pack
```

### Build and Run
```sh
git clone https://github.com/ivanceras/hackernews-sauron
cd hackernews-sauron

# Build the WASM application
wasm-pack build . --release --target web

# Serve static files (any HTTP server works)
python3 -m http.server 8080
# or: npx serve .
# or: caddy file-server
```

Navigate to http://localhost:8080

### Deploy Anywhere
Since this is now a pure static app, you can deploy to:

#### GitHub Pages (Automated)
This repo includes a GitHub Actions workflow that automatically builds and deploys to GitHub Pages:
1. Fork this repository
2. Go to **Settings** → **Pages** 
3. Set **Source** to "GitHub Actions"
4. Push to main branch - your site will be available at `https://yourusername.github.io/hackernews-sauron`

#### Other Platforms
- **Netlify** - Drag and drop the root folder
- **Vercel** - Connect your repo  
- **Any CDN** - Upload files to your preferred hosting

![Screenshot](https://raw.githubusercontent.com/ivanceras/hackernews-sauron/master/client/assets/screenshot-hn-clone.png)


[Online demo](http://66.42.53.165)

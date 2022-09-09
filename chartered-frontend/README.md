# chartered-frontend

Everything Chartered frontend lives here, this is what users will typically
interact with on a day-to-day basis on the web.

## Developing

Once you've installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of chartered-frontend:

```bash
docker build . --build-arg VITE_CHARTERED_WEB_URL=http://127.0.0.1:3000
```

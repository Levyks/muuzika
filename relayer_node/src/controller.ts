import {TemplatedApp} from "uWebSockets.js";


export function registerRoutes(app: TemplatedApp) {
    app.get("/health", (res) => {
        res.end("OK");
    });
}
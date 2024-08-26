import {SHARED_COMPRESSOR, TemplatedApp} from "uWebSockets.js";
import {client} from "./grpc.js";

export function registerWs(app: TemplatedApp) {
    type WsUserData = {
        username: string,
        roomCode: string,
    }

    app.ws<WsUserData>("/ws", {
        compression: SHARED_COMPRESSOR,
        maxPayloadLength: 1024 * 1024,
        idleTimeout: 30,
        upgrade: (res, req, context) => {
            const username = req.getHeader('muuzika-username');
            const roomCode = req.getHeader('muuzika-room-code');

            return res.upgrade(
                {username, roomCode},
                req.getHeader('sec-websocket-key'),
                req.getHeader('sec-websocket-protocol'),
                req.getHeader('sec-websocket-extensions'),
                context
            );
        },
        open: async (ws) => {
            const data = ws.getUserData();
            try {
                await client.onConnect({
                    username: data.username,
                    roomCode: data.roomCode,
                    server: {callsign: ''}
                });
                ws.subscribe(data.roomCode);
            } catch (e) {
                console.error("Error calling onConnect", e);
                ws.end(1011, "Internal server error");
                return;
            }
        },
        message: (ws, message, isBinary) => {
            console.log("message", message, isBinary);
        },
    });
}
import uWS from "uWebSockets.js";
import {ConnectionHandlerServiceClient} from "./compiled/connection_handler_pb.client.js";
import {GrpcTransport} from "@protobuf-ts/grpc-transport";
import {ChannelCredentials} from "@grpc/grpc-js";
import container from "rhea";
import {BroadcastMessage, BroadcastMessageData} from "./compiled/broadcast_pb.js";

// TODO: clean this up


// GRPC
const callsign = "relayer-" + Date.now();
const transport = new GrpcTransport({
    host: "localhost:50051",
    channelCredentials: ChannelCredentials.createInsecure(),
});

const client = new ConnectionHandlerServiceClient(transport);

// AMQP
const connection = container.connect({
    host: 'localhost',
    port: 5672,
    username: 'artemis',
    password: 'artemis',
});
const receiver = connection.open_receiver('muuzika.broadcast');

receiver.on('message', function (context) {
    const message = BroadcastMessage.fromBinary(context.message.body.content);
    if (!message.data) return;
    app.publish(message.roomCode, BroadcastMessageData.toJsonString(message.data));
});

// HTTP/WS
const app = uWS.App();

type WsUserData = {
    username: string,
    roomCode: string,
}

app.ws<WsUserData>("/ws", {
    compression: uWS.SHARED_COMPRESSOR,
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
                server: {callsign}
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

app.get("/health", (res) => {
    res.end("OK");
});


const port = Number(process.env.PORT) || 8080;
app.listen(port, (token) => {
    if (token) {
        console.log(`Listening on port ${port}`);
    } else {
        console.log(`Failed to listen on port ${port}`);
    }
});
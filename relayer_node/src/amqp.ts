import container from "rhea";
import {BroadcastMessage, BroadcastMessageData} from "../compiled/broadcast_pb.js";
import {TemplatedApp} from "uWebSockets.js";

export function registerAmqpListener(app: TemplatedApp) {
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
}
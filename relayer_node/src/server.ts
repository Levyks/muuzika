import uWS from "uWebSockets.js";
import {registerRoutes} from "./controller.js";
import {registerWs} from "./ws.js";
import {registerAmqpListener} from "./amqp.js";

const app = uWS.App();

registerRoutes(app);
registerWs(app);
registerAmqpListener(app);

const port = Number(process.env.PORT) || 8080;
app.listen(port, (token) => {
    if (token) {
        console.log(`Listening on port ${port}`);
    } else {
        console.log(`Failed to listen on port ${port}`);
        process.exit(1);
    }
});
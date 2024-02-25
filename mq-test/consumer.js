const connection = require("./connection");

let counter = 0;

/**
 * @param {import('rhea').EventContext} context
 */
function handleMessage(context) {
    const now = new Date().toISOString();
    console.log(`[${now}] [${counter++}] Received: `, context.message.body);
}

connection.on("message", handleMessage);

connection.open_receiver('jobs');

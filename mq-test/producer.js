const connection = require("./connection");

let counter = 0;

/**
 * @param {import('rhea').EventContext} context
 */
async function handleSendable(context) {
    //while (counter < 1000000) {
    const now = new Date().toISOString();
    context.sender.send({
        body: JSON.stringify({
            t: 'Log',
            c: 'Hello World',
        }),
        message_annotations: {
            //"x-opt-delivery-delay": 0,
        },
    });
    console.log(`[${now}] [${counter++}] Sent message`);
    //process.exit();
}

connection.on("sendable", handleSendable);

connection.open_sender('muuzika_jobs');

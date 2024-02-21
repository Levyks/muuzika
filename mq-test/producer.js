const connection = require("./connection");

let counter = 0;

/**
 * @param {import('rhea').EventContext} context
 */
async function handleSendable(context) {
  //while (counter < 1000000) {
  const now = new Date().toISOString();
  context.sender.send({
    body: `Message sent at ${now}`,
    message_annotations: {
      "x-opt-delivery-delay": 500000,
    },
  });
  console.log(`[${now}] [${counter++}] Sent message`);
  await new Promise((resolve) => setTimeout(resolve, 1));
  //}
}

connection.on("sendable", handleSendable);

connection.open_sender("foo");

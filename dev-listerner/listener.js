const container = require('rhea');

container.on('message', context => {
    console.log('received', context.message.body, context.message.body.content);
    const buffer = context.message.body.content;
    const str = buffer.toString('utf-8');
    console.log(str);
});

const connection = container.connect({
    host: 'localhost',
    port: 5672,
    username: 'artemis',
    password: 'artemis',
});

connection.open_receiver('muuzika.broadcast');

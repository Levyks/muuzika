const container = require("rhea");

const connection = container.connect({
  host: "localhost",
  port: 5672,
  username: "artemis",
  password: "artemis",
});

module.exports = connection;

# Muuzika

This is stack is supposed to be ridiculous, not practical or efficient at all, i'm just doing it for the fun

## Front-end of the Back-end

- Will handle the HTTP requests and Websocket connections directly from the client
- Messages will just be redirected to the appropriate service (listed below) via GRPC
- Will probably be written in Node.js, as Socket.IO is just way too good to pass up, and when using the uWS.js adapter, it's actually pretty fast/memory efficient

## Proxying / Load balancing

- For now, I'll use an NGINX server as the reverse proxy in front of all the GRPC servers.
- I plan on studying alternatives for using client-side load balancing, but I have to look at the GRPC options, preferably I would want something not JVM based.

## Services/Methods

- Lobby

  - Create room
  - Join room
  - Common
    - Captcha

- Presence

  - Connect
  - Disconnect

- Playlist

  - Manage tokens
  - Fetch playlist
  - Process/Group playlist (CPU intensive)

- Guess

  - Just guess lol

- Chat
  - Send message

### Languages

- I plan on having a single binary written in Rust with all of these services, just because it's cool, and if I ever need to scale, it will probably be the most efficient implementation.
- But just for the fun of it, I'll try to implement at least one service in the following languages:
  - Elixir
  - JVM (maybe Java, if I really feel like hating myself)
  - Go
  - Python
  - OCaml
  - Something lower level (I would like Zig, but I couldn't really find a GRPC library for it yet, so maybe C or C++)

## Worker

- These will deal with the schedule stuff (ex: End round after X seconds, calculate scores, etc)
- Will use BullMQ on top of Redis for the queueing system
- Because of that, Node.js is the obvious choice here, but they also have a non-production ready Python lib, and an even less production ready Redis module written in Zig, so if I'm feeling risky, I could pursue those options

## Database

- I'll already need Redis for the queueing system, so I'll probably use it for the database as well.

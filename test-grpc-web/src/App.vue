<script setup lang="ts">
import HelloWorld from "./components/HelloWorld.vue";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { LobbyServiceClient } from "./generated/lobby.client";

const transport = new GrpcWebFetchTransport({
  format: "binary",
  baseUrl: "http://localhost:50051",
});

const client = new LobbyServiceClient(transport);

const abortController = new AbortController();

const stream = client.listenMessages(
  {},
  {
    abort: abortController.signal,
  }
);

stream.responses.onMessage((message) => {
  console.log("message", message);
});

Object.assign(window, { client, stream, abortController });
</script>

<template>
  <div>
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo" alt="Vite logo" />
    </a>
    <a href="https://vuejs.org/" target="_blank">
      <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
    </a>
  </div>
  <HelloWorld msg="Vite + Vue" />
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>

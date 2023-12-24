import { Queue, Worker } from "bullmq";
import { Redis } from "ioredis";

async function main() {
  const redis = new Redis({
    maxRetriesPerRequest: null,
    enableReadyCheck: false,
    password: "12345",
  });
  await redis.set("foo", "bar");
  const message = await redis.get("foo");
  console.log("got", message);

  const myQueue = new Queue("foo", {
    connection: redis,
  });

  const worker = new Worker(
    "foo",
    async (job: { name: string; data: unknown }) => {
      console.log(new Date().toISOString(), "received", job.name, job.data);
    },
    {
      connection: redis,
    }
  );

  await myQueue.add("myJobName", { foo: "bar" });
  await myQueue.add("myJobName", { qux: "baz" });

  console.log("keys", myQueue.keys);
}

main();

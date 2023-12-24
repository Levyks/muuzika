FROM redis:7.2-alpine3.19

# provides necessary ld-linux-x86-64.so.2 for bullmq-redis
RUN apk add gcompat

COPY redis.conf /usr/local/etc/redis/redis.conf
COPY libbullmq__1_1_0.so /usr/local/lib/libbullmq.so

CMD ["sh", "-c", "exec redis-server /usr/local/etc/redis/redis.conf --requirepass \"$REDIS_PASSWORD\" "]
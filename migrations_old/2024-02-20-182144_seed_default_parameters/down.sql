DELETE
FROM parameters
WHERE name IN (
               'SNOWFLAKE_EPOCH_OFFSET_MS',
               'ROUND_DURATION_LEEWAY_MS',
               'MIN_USERNAME_LENGTH',
               'MAX_USERNAME_LENGTH',
               'MAX_ROUNDS',
               'MAX_PLAYERS',
               'MAX_PLAYLIST_SIZE',
               'JWT_SECRET'
    )

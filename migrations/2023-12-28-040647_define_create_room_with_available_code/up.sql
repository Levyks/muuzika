CREATE OR REPLACE FUNCTION create_room_with_available_code(
    id BIGINT
)
    RETURNS VARCHAR(31)
AS $$
DECLARE
    code VARCHAR(31);
BEGIN
    SELECT get_available_code() INTO code;

    IF code IS NULL THEN
        RAISE EXCEPTION 'OUT_OF_AVAILABLE_CODES';
    END IF;

    INSERT INTO rooms (id, code) VALUES (id, code);

    RETURN code;
END;
$$ LANGUAGE plpgsql;
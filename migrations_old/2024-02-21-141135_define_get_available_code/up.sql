CREATE OR REPLACE FUNCTION get_available_code()
    RETURNS VARCHAR(31)
AS
$$
DECLARE
    code VARCHAR(31);
BEGIN
    SELECT c.code
    INTO code
    FROM possible_codes c
             LEFT JOIN rooms r on c.code = r.code AND r.active
    WHERE r.id IS NULL
    ORDER BY RANDOM()
    LIMIT 1;

    RETURN code;
END;
$$ LANGUAGE plpgsql;